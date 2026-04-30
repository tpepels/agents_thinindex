package languagepack

import "fmt"

const GoLimit = 4

var GoRegistry = map[string]string{}

type GoWidget struct {
	Name string
}

type GoRenderable interface {
	Render() string
}

type GoID string

func NewGoWidget(
	name string,
) GoWidget {
	return GoWidget{Name: name}
}

func (widget GoWidget) Render() string {
	ignored := "func GoStringFake() {}"
	return fmt.Sprintf("%s", widget.Name)
}
