package internal

import (
	"errors"
	"fmt"
	"strconv"
)

type Ast struct {
	Assignments map[string]Node
}

type Node struct {
	Type  NodeType
	Value any
}

type NodeType int

const (
	String NodeType = iota + 1
	Integer
	Float
	Array
	Dict
	Bool
	Null = 0
)

func Parse(tokens []Token) (Ast, error) {
	ast := Ast{Assignments: map[string]Node{}}
	var EOF bool

	_len := len(tokens)

	var scopes []string
	for i := 0; i < _len; i++ {
		key := tokens[i]

		switch key.Type {
		case NewLine:
			continue
		case Dot:
			// TODO: fix variable name 'a.b = null' -> 'b = null' should return err
			return ast, fmt.Errorf("%vs are only allowed inside dictionaries", key.Type)
		case Symbol:
			EOF = i+1 >= _len
			if EOF {
				return ast, fmt.Errorf("missing declaration after symbol '%v' at %v", key, key.Position)
			}

			i++
			next := tokens[i]

			if next.Type == Equals {
				EOF = i+1 >= _len
				if EOF {
					return ast, fmt.Errorf("missing value after key '%v' at %v", key, key.Position)
				}

				i++
				value := tokens[i]

				var node Node
				var err error

				switch value.Type {
				case OpenBracket:
					node, i, err = parseArray(tokens, i+1)
				case OpenCurly:
					node, i, err = parseDict(tokens, i+1)
				case Quote:
					node, i, err = parseString(tokens, i+1)
				case Symbol:
					node, i, err = parseSymbol(tokens, i)
				default:
					err = errors.New("unreachable")
				}

				if err != nil {
					return ast, err
				}

				_len_scopes := len(scopes)
				if _len_scopes > 0 {
					scopes = append(scopes, key.Symbol)

					root := Node{
						Type:  Dict,
						Value: map[string]Node{},
					}

					child := root
					for i, scope := range scopes {
						fmt.Println(i, scope, _len_scopes)

						if i == 0 {
							continue
						}

						// _len_scopes doesn't need -1 because the last scope was added after
						if i == _len_scopes {
							child.Value.(map[string]Node)[scope] = node
						} else {
							child.Value.(map[string]Node)[scope] = Node{
								Type:  Dict,
								Value: map[string]Node{},
							}
						}
					}

					ast.Assignments[scopes[0]] = root
				} else {
					ast.Assignments[key.Symbol] = node
				}
			} else if next.Type == Colon {
				EOF = i+1 >= _len
				if EOF {
					return ast, fmt.Errorf("missing symbol ':' after '%v' at %v", key, key.Position)
				}

				i++
				next := tokens[i]

				if next.Type == Colon {
					scopes = append(scopes, key.Symbol)
					continue
				}

				return ast, fmt.Errorf("invalid symbol '%v' at %v, should be ':'", key.Symbol, key.Position)
			}
		default:
			return ast, fmt.Errorf("unreachable default tokentype '%v' at %v", key.Type, key.Position)
		}
	}

	return ast, nil
}

func parseSymbol(tokens []Token, at int) (Node, int, error) {
	node, i, err := parseNumber(tokens, at)

	if err == nil {
		return node, i, nil
	}

	return parseBool(tokens, at)
}

func parseBool(tokens []Token, at int) (Node, int, error) {
	value := tokens[at]

	v, err := strconv.ParseBool(value.Symbol)
	if err == nil {
		return Node{
			Type:  Bool,
			Value: v,
		}, at, nil
	}

	return Node{}, at, fmt.Errorf("invalid symbol '%v' at %v", value.Symbol, value.Position)
}

func parseNumber(tokens []Token, at int) (Node, int, error) {
	var EOF bool

	len := len(tokens)

	EOF = at+1 >= len
	NOT_FLOAT := tokens[at+1].Type != Dot

	// Integer
	if EOF || NOT_FLOAT {
		token := tokens[at]
		v, err := strconv.ParseInt(token.Symbol, 0, 64)

		if err == nil {
			return Node{
				Type:  Integer,
				Value: v,
			}, at, nil
		}
	}

	// Will have a dot, no need to check for it
	at++

	EOF = at+1 >= len
	MAYBE_FLOAT := tokens[at+1].Type == Symbol

	if EOF {
		return Node{}, at, fmt.Errorf("missing decimal part of float value at %v", tokens[at].Position)
	}

	if !EOF && MAYBE_FLOAT {
		at++

		left := tokens[at-2]
		right := tokens[at]

		v, err := strconv.ParseFloat(left.Symbol+"."+right.Symbol, 64)
		if err == nil {
			return Node{
				Type:  Float,
				Value: v,
			}, at, nil
		}

		return Node{}, at, fmt.Errorf("'%v.%v' not a number", left.Symbol, right.Symbol)
	}

	return Node{}, at, fmt.Errorf("'%v' not a number", tokens[at-1])
}

func parseString(tokens []Token, at int) (Node, int, error) {
	var str string

	len := len(tokens)

	for {
		token := tokens[at]

		if token.Type == Quote {
			break
		}

		if token.Type == NewLine {
			return Node{}, at, fmt.Errorf("broken string at %v", token.Position)
		}

		str += token.Symbol

		if at+1 <= len-1 {
			at++
		} else {
			return Node{}, at, fmt.Errorf("unclosed string at %v", token.Position)
		}
	}

	return Node{
		Type:  String,
		Value: str,
	}, at, nil
}

func parseArray(tokens []Token, at int) (Node, int, error) {
	array := []Node{}
	var err error

	len := len(tokens)

	for {
		token := tokens[at]
		var node Node

		if token.Type == CloseBracket {
			break
		}

		switch token.Type {
		case OpenBracket:
			node, at, err = parseArray(tokens, at+1)
		case OpenCurly:
			node, at, err = parseDict(tokens, at+1)
		case Quote:
			node, at, err = parseString(tokens, at+1)
		case Symbol:
			node, at, err = parseSymbol(tokens, at)
		}

		if node.Value != nil {
			array = append(array, node)
		}

		at++

		EOF := at >= len
		if EOF {
			return Node{}, at, fmt.Errorf("unclosed array at %v", token.Position)
		}
	}

	if err != nil {
		return Node{}, at, err
	}

	return Node{
		Type:  Array,
		Value: array,
	}, at, nil
}

func parseDict(tokens []Token, at int) (Node, int, error) {
	dict := map[string]Node{}
	var err error
	var EOF bool

	len := len(tokens)

	for {
		token := tokens[at]
		var key string
		var node Node

		if token.Type == CloseCurly {
			break
		}

		if token.Type == Dot {
			EOF = at+1 >= len
			if EOF {
				return Node{}, at, fmt.Errorf("missing variable name at %v", token.Position)
			}

			at++
			token = tokens[at]

			if token.Type == Symbol {
				key = token.Symbol

				EOF = at+1 >= len
				if EOF {
					return Node{}, at, fmt.Errorf("missing colon at %v", token.Position)
				}

				at++
				token = tokens[at]

				if token.Type == Colon {
					EOF = at+1 >= len
					if EOF {
						return Node{}, at, fmt.Errorf("missing value at %v", token.Position)
					}

					at++
					token = tokens[at]

					switch token.Type {
					case OpenBracket:
						node, at, err = parseArray(tokens, at+1)
					case OpenCurly:
						node, at, err = parseDict(tokens, at+1)
					case Quote:
						node, at, err = parseString(tokens, at+1)
					case Symbol:
						node, at, err = parseSymbol(tokens, at)
					case Colon:
						return Node{}, at, fmt.Errorf("nested declarations are not allowed inside dictionaries")
					}
				} else {
					return Node{}, at, fmt.Errorf("invalid token %v at %v expected Colon", token.Type, token.Position)
				}
			} else {
				return Node{}, at, fmt.Errorf("invalid token %v at %v, expected Symbol", token.Type, token.Position)
			}
		}

		if key != "" && node.Value != nil {
			dict[key] = node
		}

		at++
		EOF = at >= len
		if EOF {
			return Node{}, at, fmt.Errorf("unclosed dict at %v", token.Position)
		}
	}

	if err != nil {
		return Node{}, at, err
	}

	return Node{
		Type:  Dict,
		Value: dict,
	}, at, nil
}

func (ast Ast) String() string {
	str := "{\n"

	for key, node := range ast.Assignments {
		str += fmt.Sprintf("\t%v = ", key)

		switch node.Type {
		case Null:
			str += "\033[31m"
		case String:
			str += "\033[32m"
		case Integer:
			str += "\033[33m"
		case Float:
			str += "\033[33m"
		case Bool:
			str += "\033[34m"
		}

		str += fmt.Sprintf("%v\033[m\n", node.String())
	}

	str += "}"
	return str
}

func (nodeType NodeType) String() string {
	switch nodeType {
	case String:
		return "String"
	case Integer:
		return "Integer"
	case Float:
		return "Float"
	case Array:
		return "Array"
	case Dict:
		return "Dict"
	case Bool:
		return "Bool"
	default:
		return "Null"
	}
}

func (node Node) String() string {
	switch node.Type {
	case String:
		return fmt.Sprintf("\033[32m\"%s\"\033[m", node.Value)
	case Integer:
		return fmt.Sprintf("\033[33m%d\033[m", node.Value)
	case Float:
		return fmt.Sprintf("\033[33m%f\033[m", node.Value)
	case Bool:
		return fmt.Sprintf("\033[34m%v\033[m", node.Value)
	case Array:
		str := "["

		for i, value := range node.Value.([]Node) {
			if i > 0 {
				str += ", "
			}

			str += value.String()
		}

		str += "]"
		return str
	case Dict:
		str := "{"

		first := true
		for key, value := range node.Value.(map[string]Node) {
			if !first {
				str += ", "
			}

			first = false
			str += fmt.Sprintf("%s: %v", key, value.String())
		}

		str += "}"
		return str
	default:
		return "\033[31mnull\033[m"
	}
}
