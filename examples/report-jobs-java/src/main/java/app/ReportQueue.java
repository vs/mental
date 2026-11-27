package app;

import org.springframework.amqp.rabbit.core.RabbitTemplate;
import org.springframework.stereotype.Component;

@Component
public class ReportQueue {

    static final String NAME = "reports";

    private final RabbitTemplate rabbit;

    public ReportQueue(RabbitTemplate rabbit) {
        this.rabbit = rabbit;
    }

    public void delay(String reportId) {
        delay(reportId, 0);
    }

    public void delay(String reportId, int retries) {
        rabbit.convertAndSend(NAME, reportId, message -> {
            message.getMessageProperties().setHeader("retries", retries);
            return message;
        });
    }
}
