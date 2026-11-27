package app;

import java.time.Duration;
import java.util.Map;
import org.springframework.amqp.rabbit.annotation.RabbitListener;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Profile;
import org.springframework.data.domain.PageRequest;
import org.springframework.http.client.SimpleClientHttpRequestFactory;
import org.springframework.messaging.handler.annotation.Header;
import org.springframework.stereotype.Component;
import org.springframework.web.client.RestClient;
import software.amazon.awssdk.core.sync.RequestBody;
import software.amazon.awssdk.services.s3.S3Client;
import software.amazon.awssdk.services.s3.model.PutObjectRequest;
import software.amazon.awssdk.services.s3.model.S3Exception;

@Component
@Profile("worker")
public class ReportTasks {

    static final int MAX_RETRIES = 3;

    private final ReportRepository reports;
    private final OrderRepository orders;
    private final ReportQueue queue;
    private final S3Client s3 = S3Client.create();
    private final RestClient http;
    private final String bucket;
    private final String webhookUrl;

    public ReportTasks(ReportRepository reports, OrderRepository orders, ReportQueue queue,
                       @Value("${reports.bucket}") String bucket,
                       @Value("${reports.webhook-url:}") String webhookUrl) {
        this.reports = reports;
        this.orders = orders;
        this.queue = queue;
        this.bucket = bucket;
        this.webhookUrl = webhookUrl;
        var requests = new SimpleClientHttpRequestFactory();
        requests.setConnectTimeout(Duration.ofSeconds(10));
        requests.setReadTimeout(Duration.ofSeconds(10));
        this.http = RestClient.builder().requestFactory(requests).build();
    }

    @RabbitListener(queues = ReportQueue.NAME)
    public void generateReport(String reportId, @Header("retries") int retries) {
        var report = reports.findById(reportId).orElse(null);
        if (report == null || report.getStatus() != ReportStatus.PENDING) {
            return;
        }
        report.setStatus(ReportStatus.RUNNING);
        reports.save(report);

        var csv = new StringBuilder();
        for (int page = 0; ; page++) {
            var batch = orders.findByCreatedBetweenOrderById(report.getStart(), report.getEnd(),
                                                             PageRequest.of(page, 1000));
            if (batch.isEmpty()) {
                break;
            }
            for (var order : batch) {
                csv.append(order.getId()).append(',').append(order.getCreated()).append(',')
                   .append(order.getTotalCents()).append('\n');
            }
        }

        var key = "reports/" + report.getId() + ".csv";
        try {
            s3.putObject(PutObjectRequest.builder().bucket(bucket).key(key).build(),
                         RequestBody.fromString(csv.toString()));
        } catch (S3Exception e) {
            report.setStatus(retries >= MAX_RETRIES ? ReportStatus.FAILED : ReportStatus.PENDING);
            reports.save(report);
            if (report.getStatus() == ReportStatus.PENDING) {
                queue.delay(reportId, retries + 1);
            }
            return;
        }

        report.setStatus(ReportStatus.DONE);
        report.setUrl("s3://" + bucket + "/" + key);
        reports.save(report);

        if (!webhookUrl.isEmpty()) {
            http.post().uri(webhookUrl).body(Map.of("id", reportId, "status", "DONE"))
                .retrieve().toBodilessEntity();
        }
    }
}
