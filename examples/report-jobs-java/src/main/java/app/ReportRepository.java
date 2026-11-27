package app;

import java.time.Instant;
import java.util.List;
import java.util.Optional;
import org.springframework.data.jpa.repository.JpaRepository;

public interface ReportRepository extends JpaRepository<Report, String> {

    Optional<Report> findByIdempotencyKey(String idempotencyKey);

    List<Report> findByStatusAndUpdatedAtBefore(ReportStatus status, Instant cutoff);
}
