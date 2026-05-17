package main

import (
	"crypto/rand"
	"crypto/sha256"
	"fmt"
	"net"
	"os"
	"time"

	"github.com/flynn/noise"
)

func findServerByBeacon() (string, error) {
	addr := net.UDPAddr{Port: 9999, IP: net.ParseIP("0.0.0.0")}
	conn, _ := net.ListenUDP("udp", &addr)
	defer conn.Close()
	conn.SetReadDeadline(time.Now().Add(15 * time.Second))

	buf := make([]byte, 1024)
	fmt.Println("📡 Ожидание сигнала сервера...")
	n, remoteAddr, err := conn.ReadFromUDP(buf)
	if err != nil {
		return "", err
	}

	if string(buf[:n]) == "NOISE_SERVER_8080" {
		return remoteAddr.IP.String() + ":8080", nil
	}
	return "", fmt.Errorf("неверный сигнал")
}

func main() {
	serverAddr, err := findServerByBeacon()
	if err != nil {
		fmt.Fprintf(os.Stderr, "❌ Сервер не найден: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("✅ Сервер найден: %s\n", serverAddr)

	conn, err := net.Dial("tcp", serverAddr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "❌ Не удалось подключиться: %v\n", err)
		os.Exit(1)
	}
	defer conn.Close()

	psk := sha256.Sum256([]byte("mylittle-r1xe<3"))
	keyID, _ := noise.DH25519.GenerateKeypair(rand.Reader)
	hs, _ := noise.NewHandshakeState(noise.Config{
		CipherSuite: noise.NewCipherSuite(noise.DH25519, noise.CipherAESGCM, noise.HashSHA256),
		Pattern:     noise.HandshakeXX, Initiator: true, PresharedKey: psk[:], StaticKeypair: keyID, Prologue: []byte("demo-app-v1"),
	})

	buf := make([]byte, 2048)
	msg, _, _, _ := hs.WriteMessage(nil, nil)
	conn.Write(msg)
	n, _ := conn.Read(buf)
	hs.ReadMessage(nil, buf[:n])
	msg, send, recv, _ := hs.WriteMessage(nil, nil)
	conn.Write(msg)

	fmt.Println("🔐 Noise handshake завершён")

	c, _ := send.Encrypt(nil, nil, []byte("shutdown\n"))
	conn.Write(c)
	fmt.Println("📤 Команда shutdown отправлена")

	n, _ = conn.Read(buf)
	resp, _ := recv.Decrypt(nil, nil, buf[:n])
	fmt.Printf("✅ Ответ: %s", string(resp))

	fmt.Println("✨ Клиент завершается")
}
