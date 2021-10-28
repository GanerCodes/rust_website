import socket, time

s = socket.socket()
s.connect(("127.0.0.1", 443))
i = 0
while 1:
    s.send(f"haha yes {i}".encode('utf-8'))
    print(s.recv(1024))
    i += 1
    time.sleep(1)