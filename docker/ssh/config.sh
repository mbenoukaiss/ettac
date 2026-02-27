#!/bin/sh

if [ -z "$USERNAME" ]; then
  echo "You must define USERNAME to start SSH server"
  exit 1
fi

if [ -z "$PASSWORD" ] && [ -z "$PUBLIC_KEY" ]; then
  echo "Either define PASSWORD or PUBLIC_KEY to start SSH server"
  exit 2
fi

ARGS="-o AddressFamily=inet -o GatewayPorts=yes -o AllowAgentForwarding=yes -o AllowTcpForwarding=yes"

if [ -n "$PERMIT_ROOT_LOGIN" ]; then
    ARGS="$ARGS -o PermitRootLogin=$PERMIT_ROOT_LOGIN"
fi

if [ -n "$PASSWORD_AUTHENTICATION" ]; then
    ARGS="$ARGS -o PasswordAuthentication=$PASSWORD_AUTHENTICATION"
fi

if [ "$USERNAME" != "root" ]; then
    HOME=/home/$USERNAME
    addgroup -S $USERNAME 2>/dev/null
    adduser -S $USERNAME -G $USERNAME -h $HOME -s /bin/sh
else
    HOME=/root
fi

if [ -n "$PASSWORD" ]; then
    echo "$USERNAME:$PASSWORD" | chpasswd 2>/dev/null
else
    echo "$USERNAME:!" | chpasswd 2>/dev/null

    mkdir -p $HOME/.ssh
    echo "$PUBLIC_KEY" > $HOME/.ssh/authorized_keys

    chown -R $USERNAME:$USERNAME $HOME/.ssh
    chmod 700 $HOME/.ssh
    chmod 600 $HOME/.ssh/authorized_keys
fi

/usr/sbin/sshd -D $ARGS
