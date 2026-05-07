package greeter

type Greeter struct {
	Name string
}

type Renderer interface {
	Render() string
}

var Registry = map[string]Greeter{}

func NewGreeter(name string) Greeter {
	return Greeter{Name: name}
}

func (greeter Greeter) Render() string {
	ignored := "func GoSyntheticStringFake() {}"
	return ignored[:0] + greeter.Name
}

// func GoSyntheticCommentFake() {}
