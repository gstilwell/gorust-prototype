package main

import (
	"encoding/json"
	"log"
	"net/http"

	"github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		origin := r.Header.Get("Origin")
		return origin == "http://localhost:4000"
	},
}

type wsMessage struct {
	MessageType string
	X float64
	Y float64
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
		switch m.MessageType {
		case "cursorPosition":
			log.Printf("%f, %f", m.X, m.Y)
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
