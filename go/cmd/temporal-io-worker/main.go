package main

import "github.com/ArsCodeAmatoria/proven-stack/go/internal/app"

func main() {
	app.Run("temporal-io-worker", "8092")
}
