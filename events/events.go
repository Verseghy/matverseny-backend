package events

import (
	"sync"
)

type Events struct {
	lock            sync.Mutex
	timeSubscribers []*TimeSubscriber
}

var e *Events

func ensureEvents() {
	if e == nil {
		e = &Events{}
	}
}

func ConsumeProblems(isAdmin bool) {
	ensureEvents()

}

func ConsumeSolutions(team string) {
	ensureEvents()

}
