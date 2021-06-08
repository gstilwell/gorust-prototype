package hub

import "github.com/gorilla/websocket"

type ClientConnection struct {
	conn *websocket.Conn
}

func (cc *ClientConnection) connect() {

}
