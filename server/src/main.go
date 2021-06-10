package main

import (
	"encoding/json"
	"log"
	"math/rand"
	"net/http"
	"time"

	"github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		origin := r.Header.Get("Origin")
		return origin == "http://localhost:4000"
	},
}

type WelcomeMessage struct {
	MessageType string
	ClientId uint32
}

func (m *WelcomeMessage) create(clientId uint32) {
	m.MessageType = "welcome"
	m.ClientId = clientId
}

type wsMessage struct {
	MessageType string
	X float64
	Y float64
	ClientId uint32
}

func websocketConnect(w http.ResponseWriter, r *http.Request) {
	c, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Print("upgrade error:", err)
		return
	}
	defer c.Close()
	for {
		_, message, err := c.ReadMessage()
		if err != nil {
			log.Println("read error:", err)
			break
		}

		var m wsMessage
		err = json.Unmarshal(message, &m)
		if err != nil {
			log.Println("Unmarshal error:", err)
			break
		}

		log.Printf("%s", m.MessageType)
		switch m.MessageType {
		case "cursorPosition":
			log.Printf("id: %d -- %f, %f", m.ClientId, m.X, m.Y)
		case "salutations":
			rand.Seed(time.Now().UnixNano())
			clientId := rand.Uint32()
			log.Printf("hello, %d", clientId)

			response := WelcomeMessage{}
			response.create(clientId)
			//jsonResponse, err := json.Marshal(&response)
			if err != nil {
				log.Println("welcome error:", err)
			}
			c.WriteJSON(response)
		case "ack":
			log.Printf("got ack from %d", m.ClientId)
		default:
			log.Printf("got unknown message %s", m.MessageType)
		}

		if err != nil {
			log.Println("write error:", err)
			break
		}
	}
}

func main() {
	http.HandleFunc("/websocket", websocketConnect)
	http.ListenAndServe(":5000", nil)
}
