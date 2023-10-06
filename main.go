package main

import (
	"log"
	"os"

	internal "github.com/felpofo/kfg/internal"
)

type MyLogger struct {
	*log.Logger
}

func (wrapper MyLogger) Log(msg string, file string) {
	wrapper.Logger.Printf("%v '\x1b[33m%v\x1b[m'", msg, file)
}

func main() {
	Debug := MyLogger{log.New(os.Stderr, "\x1b[35mDEBUG\x1b[m :: ", 0)}
	Info := MyLogger{log.New(os.Stderr, "\x1b[32mINFO\x1b[m :: ", 0)}
	Error := MyLogger{log.New(os.Stderr, "\x1b[31mERROR\x1b[m :: ", 0)}

	if len(os.Args) < 2 {
		Error.Fatal("Syntax: <file>")
	}

	_, debug := os.LookupEnv("DEBUG")
	if debug {
		Debug.Print("DEBUG MODE")
	}

	file := os.Args[1]

	Info.Log("Reading", file)
	content, err := os.ReadFile(file)

	if err != nil {
		Error.Fatal(err)
	}

	Info.Log("Tokenizing", file)
	tokens := internal.Tokenize(content)

	if debug {
		for _, token := range tokens {
			Info.Printf("\033[36m%v\033[m \033[33m%#v\033[m at %v", token.Type, token.Symbol, token.Position)
		}
	}
}
