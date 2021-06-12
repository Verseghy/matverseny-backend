package events

import (
	"context"
	"github.com/google/uuid"
)

type SolutionSubscriber struct {
	ID   uuid.UUID
	Team string
	Ch   chan<- *SolutionEvent
}

type SolutionType uint32

const (
	SChange SolutionType = iota
	SDelete
)

type SolutionEvent struct {
	Type      SolutionType
	ProblemID string
	Team      string
	Value     int64
}

func ConsumeSolution(ctx context.Context, team string) <-chan *SolutionEvent {
	ensureEvents()

	ch := make(chan *SolutionEvent)
	e.lock.Lock()
	defer e.lock.Unlock()

	ID, err := uuid.NewUUID()
	if err != nil {
		panic(err)
	}
	e.solutionSubscribers = append(e.solutionSubscribers, &SolutionSubscriber{ID: ID, Ch: ch, Team: team})
	go func() {
		<-ctx.Done()
		e.lock.Lock()
		defer e.lock.Unlock()

		for k, v := range e.solutionSubscribers {
			if v.ID == ID {
				a := e.solutionSubscribers
				a[k] = a[len(a)-1]
				a[len(a)-1] = nil
				a = a[:len(a)-1]
				break
			}
		}
	}()

	return ch
}

func PublishSolution(event *SolutionEvent) {
	ensureEvents()

	e.lock.Lock()
	defer e.lock.Unlock()

	for _, v := range e.solutionSubscribers {
		if v.Team == event.Team {
			v.Ch <- event
		}
	}
}
