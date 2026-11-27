package app;

import java.time.Duration;
import java.time.Instant;
import org.springframework.context.annotation.Profile;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Component;

@Component
@Profile("beat")
public class ReportBeat {

    private final ReportRepository reports;
    private final ReportQueue queue;

    public ReportBeat(ReportRepository reports, ReportQueue queue) {
        this.reports = reports;
        this.queue = queue;
    }

    @Scheduled(fixedRate = 300_000)
    public void requeueStuck() {
        var cutoff = Instant.now().minus(Duration.ofMinutes(15));
        var stuck = reports.findByStatusAndUpdatedAtBefore(ReportStatus.RUNNING, cutoff);
        for (var report : stuck) {
            report.setStatus(ReportStatus.PENDING);
        }
        reports.saveAll(stuck);
        for (var report : stuck) {
            queue.delay(report.getId());
        }
    }
}
