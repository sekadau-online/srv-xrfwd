# srv-xrfwd
server fowarder
# Gunakan aplikasi client side yang sudah dibuat sebelumnya
# Atau gunakan SSH command langsung
ssh -R 8080:localhost:3000 user@server-ip -p 2222

http://server-ip:8080
http://server-ip:8080/clients
http://server-ip:8080/status
