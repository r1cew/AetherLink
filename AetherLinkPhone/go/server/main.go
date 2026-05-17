package main

import (
	"crypto/rand"
	"crypto/sha256"
	"fmt"
	"net"
	"os/exec"
	"runtime"
	"strings"
	"time"

	"github.com/flynn/noise"
)

func startBeacon() {
	for {
		ifaces, _ := net.Interfaces()
		for _, iface := range ifaces {
			addrs, _ := iface.Addrs()
			for _, addr := range addrs {
				if ipnet, ok := addr.(*net.IPNet); ok && !ipnet.IP.IsLoopback() && ipnet.IP.To4() != nil {
					ipStr := ipnet.IP.String()

					if strings.HasPrefix(ipStr, "192.168.") {

						broadcast := make(net.IP, len(ipnet.IP.To4()))
						for i := range ipnet.IP.To4() {
							broadcast[i] = ipnet.IP.To4()[i] | ^ipnet.Mask[i]
						}

						conn, err := net.DialUDP("udp", nil, &net.UDPAddr{IP: broadcast, Port: 9999})
						if err == nil {
							conn.Write([]byte("NOISE_SERVER_8080"))
							conn.Close()
						}
					}
				}
			}
		}
		time.Sleep(5 * time.Second)
	}
}

func shutdownPC() {
	var cmd *exec.Cmd

	if runtime.GOOS == "windows" {
		cmd = exec.Command("shutdown", "/s", "/t", "0", "/f")
	} else if runtime.GOOS == "linux" {
		cmd = exec.Command("sudo", "shutdown", "-h", "now")
	} else if runtime.GOOS == "darwin" {
		cmd = exec.Command("osascript", "-e", "tell application \"System Events\" to shut down")
	}

	if cmd != nil {
		err := cmd.Run()
		if err != nil {
			fmt.Printf("❌ Ошибка выключения: %v\n", err)
		} else {
			fmt.Println("🔴 ПК выключается...")
		}
	}
}

func main() {
	go startBeacon()
	listener, err := net.Listen("tcp", ":8080")
	if err != nil {
		fmt.Println("❌ Ошибка запуска сервера:", err)
		return
	}
	fmt.Println("🚀 Сервер готов, маяк запущен...")

	for {
		conn, _ := listener.Accept()
		go handleClient(conn)
	}
}

func handleClient(conn net.Conn) {
	defer conn.Close()
	psk := sha256.Sum256([]byte("mylittle-r1xe<3"))
	keyID, _ := noise.DH25519.GenerateKeypair(rand.Reader)
	hs, _ := noise.NewHandshakeState(noise.Config{
		CipherSuite: noise.NewCipherSuite(noise.DH25519, noise.CipherAESGCM, noise.HashSHA256),
		Pattern:     noise.HandshakeXX, Initiator: false, PresharedKey: psk[:], StaticKeypair: keyID, Prologue: []byte("demo-app-v1"),
	})

	buf := make([]byte, 2048)
	n, _ := conn.Read(buf)
	_, _, _, err := hs.ReadMessage(nil, buf[:n])
	if err != nil {
		return
	}

	msg, _, _, _ := hs.WriteMessage(nil, nil)
	conn.Write(msg)

	n, _ = conn.Read(buf)
	_, recv, send, err := hs.ReadMessage(nil, buf[:n])
	if err != nil {
		return
	}

	fmt.Println("✅ Клиент подключен")
	n, _ = conn.Read(buf)
	plaintext, _ := recv.Decrypt(nil, nil, buf[:n])

	command := strings.TrimSpace(string(plaintext))
	fmt.Printf("📥 Получено: %s\n", command)

	if command == "shutdown" {
		fmt.Println("⚡ Команда shutdown получена!")
		resp, _ := send.Encrypt(nil, nil, []byte("shutting down\n"))
		conn.Write(resp)

		shutdownPC()
	} else {
		resp, _ := send.Encrypt(nil, nil, []byte("unknown command\n"))
		conn.Write(resp)
	}
}
