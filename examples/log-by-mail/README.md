# log4j as a sample SMTP client

By default, `log4j` is configured to send email messages to SMTP server running on .

## How To

### How To run SMTP server

```shell
docker run -p 1080:80 -p 1025:25 djfarrelly/maildev
```

which will start https://github.com/maildev/maildev SMTP server

### How To run SMTP client (log4j)

```shell
./mvnw compile exec:java
```

which will start java app configured to send emails with errors to `localhost:10000`.

Use [MailDev UI](http://localhost:1080) to verify that emails are indeed arriving at SMTP server.

### How To change configuration of SMTP client (log4j)

src/resources/log4j2.xml
```xml
...
<SMTP name="mail"
      subject="Exception in Java code"
      from="from@HOME"
      to="to@WORLD"
      smtpHost="127.0.0.1"
      smtpPort="10000"
      bufferSize="100">
</SMTP>
...
```
