FROM alpine:3.23

RUN apk add --no-cache openssh && ssh-keygen -A && echo "" > /etc/motd
COPY config.sh /

EXPOSE 22

ENTRYPOINT ["sh", "/config.sh"]
