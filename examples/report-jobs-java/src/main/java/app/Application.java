package app;

import org.springframework.amqp.core.Queue;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.annotation.Bean;
import org.springframework.scheduling.annotation.EnableScheduling;

@SpringBootApplication
@EnableScheduling
public class Application {

    @Bean
    Queue reportsQueue() {
        return new Queue(ReportQueue.NAME);
    }

    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}
