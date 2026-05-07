package main

import "example.com/synthetic/internal/greeter"
import "fmt"

const MainMode = "compact"

func main() {
	fmt.Println(greeter.NewGreeter("Ada").Render())
}
