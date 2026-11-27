package app;

import java.time.LocalDate;
import java.time.temporal.ChronoUnit;
import java.util.UUID;
import org.springframework.amqp.AmqpException;
import org.springframework.context.annotation.Profile;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestHeader;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.server.ResponseStatusException;

@RestController
@Profile("api")
public class ReportController {

    record ReportRequest(LocalDate start, LocalDate end) {
    }

    record ReportView(String id, String status, String url) {
    }

    private final ReportRepository reports;
    private final ReportQueue queue;

    public ReportController(ReportRepository reports, ReportQueue queue) {
        this.reports = reports;
        this.queue = queue;
    }

    @PostMapping("/reports")
    @ResponseStatus(HttpStatus.ACCEPTED)
    public ReportView createReport(@RequestBody ReportRequest req,
                                   @RequestHeader("Idempotency-Key") String idempotencyKey) {
        if (ChronoUnit.DAYS.between(req.start(), req.end()) > 366) {
            throw new ResponseStatusException(HttpStatus.BAD_REQUEST, "date range longer than 366 days");
        }
        var existing = reports.findByIdempotencyKey(idempotencyKey).orElse(null);
        if (existing != null) {
            return new ReportView(existing.getId(), existing.getStatus().name(), null);
        }
        var report = new Report(UUID.randomUUID().toString(), idempotencyKey,
                                req.start(), req.end(), ReportStatus.PENDING);
        reports.save(report);
        try {
            queue.delay(report.getId());
        } catch (AmqpException e) {
            throw new ResponseStatusException(HttpStatus.SERVICE_UNAVAILABLE, "queue unavailable");
        }
        return new ReportView(report.getId(), "PENDING", null);
    }

    @GetMapping("/reports/{reportId}")
    public ReportView getReport(@PathVariable String reportId) {
        var report = reports.findById(reportId).orElse(null);
        if (report == null) {
            throw new ResponseStatusException(HttpStatus.NOT_FOUND, "not found");
        }
        return new ReportView(report.getId(), report.getStatus().name(), report.getUrl());
    }
}
