Nama : Gabriel S.A.Fenanlampir
NPM: 2306315516
README TOTORIAL MODUL10;



Tutor 1:
![Screenshot Eksperimen 1.2](SS/Tutor1.png)
Refleksi:
Asynchronous programming memungkinkan unit kerja berjalan terpisah dari main thread aplikasi dan memberikan notifikasi saat selesai. Dalam kasus ini, saat spawn dipanggil, tugas tersebut segera dimulai namun tidak memblokir eksekusi kode selanjutnya. Sistem tidak menunggu timer selama 2 detik selesai untuk menjalankan perintah cetak "hey hey". Hal ini menunjukkan bahwa alur kontrol dapat berpindah untuk melakukan pekerjaan lain sementara operasi asinkron sedang berjalan. Dengan model ini, aplikasi tetap responsif meskipun ada tugas yang membutuhkan waktu tunggu lama seperti akses I/O atau timer.

Tutor 1:
![Screenshot tutor 2](SS/Tutor2.1.png)
![Screenshot tutor 2](SS/Tutor2.2.png)
![Screenshot tutor 2](SS/Tutor2.3.png)
refleksi:
Tutorial 2: Broadcast Chat
2.2. Modifying the Port
Pada eksperimen ini, saya melakukan modifikasi port koneksi pada aplikasi broadcast chat dari port bawaan menjadi port 7878 (seperti yang terlihat pada tangkapan layar eksekusi). Perubahan ini mengharuskan pembaruan pada dua sisi aplikasi, yaitu pada sisi server melalui TcpListener::bind("127.0.0.1:7878") dan pada sisi klien melalui ClientBuilder::from_uri(...). Modifikasi ini memberikan pemahaman mendasar bahwa komunikasi berbasis WebSocket tetap bertumpu pada protokol TCP yang mewajibkan klien dan server menyepakati alamat IP dan port yang spesifik agar handshake dapat terjadi. Selain itu, berkat penggunaan arsitektur asinkron dengan runtime Tokio, server dapat membuka satu port ini untuk menerima dan mengelola ribuan koneksi klien secara konkuren tanpa hambatan, jauh lebih efisien dibandingkan model thread-per-connection tradisional yang memakan banyak memori.

2.3. Small Changes: Add IP and Port
Pada bagian ini, saya menambahkan fitur identifikasi pengirim dengan menyematkan informasi alamat IP dan nomor port spesifik pengirim pada setiap pesan yang disebarkan (broadcast) oleh server. Implementasinya dilakukan dengan mengekstrak data SocketAddr saat klien pertama kali terhubung melalui listener.accept(), lalu memformatnya bersama pesan asli menggunakan makro format!("{}: {}", addr, text) sebelum dikirim ke channel broadcast. Eksperimen ini mendemonstrasikan keunggulan utama protokol WebSocket yang mendukung komunikasi dua arah (full-duplex) secara persisten. Berbeda dengan mekanisme polling pada HTTP biasa, di sini server bertindak aktif untuk langsung mendorong (push) pesan baru ke seluruh klien secara real-time. Keberadaan makro tokio::select! di dalam event loop juga sangat krusial, karena memastikan aliran stream dari klien dan antrean channel server dapat berjalan beriringan tanpa saling memblokir (non-blocking).


Tutorial 3:
![Screenshot tutor 2](SS/Tutor3.1.png)



Refleksi : 
Pada Tutorial 3 ini, saya membangun antarmuka klien berbasis web menggunakan framework Yew yang dikompilasi menjadi WebAssembly (Wasm). Berbeda dengan klien Command Line Interface (CLI) pada tutorial sebelumnya, klien Yew ini beroperasi langsung di dalam browser pengguna namun tetap memanfaatkan bahasa Rust secara penuh (full-stack Rust).

Arsitektur dan Asinkronitas:
Klien WebAssembly ini terhubung ke server obrolan lokal melalui protokol WebSocket menggunakan WebSocket::open("ws://127.0.0.1:8080"). Komunikasi asinkron di sini dikelola dengan sangat baik oleh framework Yew; setiap kali ada pesan baru yang disiarkan (broadcast) oleh server, event listener pada WebSocket akan menangkap pesan tersebut dan memperbarui state komponen menggunakan use_state. Pembaruan state ini memicu Yew untuk melakukan re-render secara reaktif pada Virtual DOM hanya pada bagian chat history yang relevan. Mekanisme ini memastikan halaman web tidak perlu dimuat ulang (refresh) dan UI tidak mengalami kondisi blocking (hang) saat menunggu pesan masuk dari pengguna lain.


Aspek Kreativitas (UI/UX):
Sebagai bentuk eksplorasi kreativitas pada antarmuka, saya mengimplementasikan desain Modern Dark Mode menggunakan modifikasi struktur makro html! dan CSS khusus. Pemilihan palet warna gelap (seperti background bernuansa charcoal atau slate) bertujuan untuk memberikan kenyamanan visual (mengurangi kelelahan mata/ eye strain) yang sangat relevan untuk aplikasi berbasis teks seperti web chat. Selain itu, tata letak elemen input, tombol pengiriman, serta chat bubble telah diatur sedemikian rupa agar responsif dan memberikan feedback visual yang jelas saat pengguna berinteraksi.