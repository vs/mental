package app;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.Id;
import jakarta.persistence.Table;
import java.time.LocalDate;

@Entity
@Table(name = "orders")
public class Order {

    @Id
    private Integer id;

    @Column(nullable = false)
    private LocalDate created;

    @Column(nullable = false)
    private Integer totalCents;

    protected Order() {
    }

    public Integer getId() {
        return id;
    }

    public LocalDate getCreated() {
        return created;
    }

    public Integer getTotalCents() {
        return totalCents;
    }
}
