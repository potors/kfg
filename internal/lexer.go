package internal

import "fmt"

type Token struct {
	Type     TokenType
	Symbol   string
	Position TokenPosition
}

type TokenType uint

const (
	Dot TokenType = iota + 1
	Comma
	Colon
	Quote
	Slash
	Asterisk
	Space
	Equals
	NewLine
	OpenBracket
	CloseBracket
	OpenCurly
	CloseCurly
	Symbol = 0
)

type TokenPosition struct {
	Line      int
	Character int
	Length    int
}

func Tokenize(buffer []byte) []Token {
	var tokens []Token
	var line, character int = 1, 0

	len := len(buffer)

	var token Token
	for i := 0; i < len; i++ {
		char := buffer[i]

		if char == '\n' {
			line++
			character = 0
		}

		token.Position = TokenPosition{
			Line:      line,
			Character: character,
			Length:    0,
		}

		token.Type = TokenTypeFrom(char)

		if token.Type == Symbol {
			token.Symbol = ""

			for {
				token.Symbol += string(buffer[i])

				if i+1 < len {
					if TokenTypeFrom(buffer[i+1]) != Symbol {
						break
					}
				}

				character++
				i++

				// EOF without \n
				// should be impossible to go after len, but... why not?
				if i >= len {
					break
				}
			}
		} else {
			token.Symbol = string(char)
			character++
		}

		token.Position.Length = character - token.Position.Character
		tokens = append(tokens, token)
	}

	return tokens
}

func Lex(tokens []Token) []Token {
	var array []Token

	isComment, isCommentBlock, isString := false, false, false

	len := len(tokens)

	for i, token := range tokens {
		var prev, next Token

		if i > 1 {
			prev = tokens[i-1]
		}

		if i < len-1 {
			next = tokens[i+1]
		}

		// Remove comments (//, /*)
		if isComment {
			if token.Type == NewLine {
				isComment = false
			}

			continue
		} else if isCommentBlock {
			if token.Type == Slash && prev.Type == Asterisk && tokens[i-2].Type != Slash {
				isCommentBlock = false
			}

			continue
		} else if token.Type == Slash {
			if next.Type == Slash {
				isComment = true
			} else if next.Type == Asterisk {
				isCommentBlock = true
			}

			continue
		}

		// Remove spaces (except inside strings)
		if !isString && token.Type != Space {
			array = append(array, token)
		}
	}

	return array
}

func TokenTypeFrom(char byte) TokenType {
	switch char {
	case '.':
		return Dot
	case ',':
		return Comma
	case ':':
		return Colon
	case '\'':
		return Quote
	case '/':
		return Slash
	case '*':
		return Asterisk
	case ' ':
		return Space
	case '=':
		return Equals
	case '\n':
		return NewLine
	case '[':
		return OpenBracket
	case ']':
		return CloseBracket
	case '{':
		return OpenCurly
	case '}':
		return CloseCurly
	default:
		return Symbol
	}
}

func (tokenType TokenType) String() string {
	switch tokenType {
	case Dot:
		return "Dot"
	case Comma:
		return "Comma"
	case Colon:
		return "Colon"
	case Quote:
		return "Quote"
	case Slash:
		return "Slash"
	case Asterisk:
		return "Asterisk"
	case Space:
		return "Space"
	case Equals:
		return "Equals"
	case NewLine:
		return "NewLine"
	case OpenBracket:
		return "OpenBracket"
	case CloseBracket:
		return "CloseBracket"
	case OpenCurly:
		return "OpenCurly"
	case CloseCurly:
		return "CloseCurly"
	default:
		return "Symbol"
	}
}

func (tokenPosition TokenPosition) String() string {
	return fmt.Sprintf("%d:%d-%d", tokenPosition.Line, tokenPosition.Character, tokenPosition.Length)
}
