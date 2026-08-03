package main

import "github.com/ArsCodeAmatoria/proven-stack/go/internal/app"

func main() {
	app.Run("analytics-worker", "8094")
}
