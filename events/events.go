package events

import (
	"sync"
)

type Events struct {
	lock                sync.Mutex
	timeSubscribers     []*TimeSubscriber
	solutionSubscribers []*SolutionSubscriber
	problemSubscribers  []*ProblemSubscriber
}

var e *Events

func ensureEvents() {
	if e == nil {
		e = &Events{}
	}
}
