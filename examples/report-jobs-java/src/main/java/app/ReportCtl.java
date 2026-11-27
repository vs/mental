package app;

import org.springframework.boot.CommandLineRunner;
import org.springframework.context.annotation.Profile;
import org.springframework.stereotype.Component;
import picocli.CommandLine;
import picocli.CommandLine.Command;
import picocli.CommandLine.Parameters;

@Component
@Profile("cli")
@Command(name = "reportctl")
public class ReportCtl implements CommandLineRunner {

    private final ReportRepository reports;
    private final ReportQueue queue;

    public ReportCtl(ReportRepository reports, ReportQueue queue) {
        this.reports = reports;
        this.queue = queue;
    }

    @Override
    public void run(String... args) {
        System.exit(new CommandLine(this).execute(args));
    }

    @Command(name = "retry")
    int retry(@Parameters(paramLabel = "REPORT_ID") String reportId) {
        var report = reports.findById(reportId).orElse(null);
        if (report == null) {
            System.err.println("Error: no such report");
            return 1;
        }
        if (report.getStatus() != ReportStatus.FAILED) {
            System.err.println("Error: report is " + report.getStatus().name() + ", not FAILED");
            return 1;
        }
        report.setStatus(ReportStatus.PENDING);
        reports.save(report);
        queue.delay(reportId);
        System.out.println("requeued");
        return 0;
    }
}
