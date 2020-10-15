# Envoy SMTP Filter

## How To

### How To install GetEnvoy CLI

```shell
curl -L https://getenvoy.io/cli | bash -s -- -b /usr/local/bin
```

For more information see https://www.getenvoy.io/install/

### How To Build

```shell
getenvoy extension build
```

### How to Run unit tests

```shell
getenvoy extension test
```

### How to Run example Envoy setup

#### Start SMTP server

```shell
docker run -p 1080:80 -p 1025:25 djfarrelly/maildev
```

which will start https://github.com/maildev/maildev SMTP server

#### Start SMTP client (log4j)

```shell
sh -c 'cd examples/log-by-mail && ./mvnw compile exec:java'
```

#### Start Envoy w/ SMTP filter

```shell
getenvoy extension run
```

And then follow instructions at [./.getenvoy/extension/examples/default/README.md](./.getenvoy/extension/examples/default/README.md)

### Example metrics

```shell
curl -s localhost:9901/stats | grep smtp
```

```shell
smtp.command.DATA.replies.positive.total: 22
smtp.command.DATA.replies.total: 22
smtp.command.DATA.reply.354.total: 22
smtp.command.DATA.total: 22
smtp.command.EHLO.replies.positive.total: 22
smtp.command.EHLO.replies.total: 22
smtp.command.EHLO.reply.250.total: 22
smtp.command.EHLO.total: 22
smtp.command.MAIL.replies.positive.total: 22
smtp.command.MAIL.replies.total: 22
smtp.command.MAIL.reply.250.total: 22
smtp.command.MAIL.total: 22
smtp.command.QUIT.replies.positive.total: 22
smtp.command.QUIT.replies.total: 22
smtp.command.QUIT.reply.221.total: 22
smtp.command.QUIT.total: 22
smtp.command.RCPT.replies.positive.total: 22
smtp.command.RCPT.replies.total: 22
smtp.command.RCPT.reply.250.total: 22
smtp.command.RCPT.total: 22
smtp.commands.replies.negative.total: 0
smtp.commands.replies.positive.total: 110
smtp.commands.replies.total: 110
smtp.commands.total: 110
smtp.connections.parse_errors.total: 0
smtp.connections.total: 22
smtp.connects.replies.negative.total: 0
smtp.connects.replies.positive.total: 22
smtp.connects.replies.total: 22
smtp.connects.reply.220.total: 22
smtp.connects.total: 22
smtp.mails.rejected.total: 0
smtp.mails.sent.total: 22
smtp.mails.total: 22
smtp.transactions.commits.replies.negative.total: 0
smtp.transactions.commits.replies.positive.total: 22
smtp.transactions.commits.replies.total: 22
smtp.transactions.commits.reply.250.total: 22
smtp.transactions.commits.total: 22
```
