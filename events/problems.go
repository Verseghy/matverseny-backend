package events

import (
	"context"
	"github.com/google/uuid"
	"matverseny-backend/entity"
)

type ProblemSubscriber struct {
	ID uuid.UUID
	Ch chan<- *ProblemEvent
}

type ProblemType uint32

const (
	PChange ProblemType = iota
	PDelete
	PSwap
	PCreate
)

type ProblemEvent struct {
	Type    ProblemType
	Problem *entity.Problem
	A       *entity.Problem
	B       *entity.Problem
	At      uint32
}

func ConsumeProblem(ctx context.Context) <-chan *ProblemEvent {
	ensureEvents()

	ch := make(chan *ProblemEvent)
	e.lock.Lock()
	defer e.lock.Unlock()

	ID, err := uuid.NewUUID()
	if err != nil {
		panic(err)
	}
	e.problemSubscribers = append(e.problemSubscribers, &ProblemSubscriber{ID: ID, Ch: ch})
	go func() {
		<-ctx.Done()
		e.lock.Lock()
		defer e.lock.Unlock()

		for k, v := range e.problemSubscribers {
			if v.ID == ID {
				a := e.problemSubscribers
				a[k] = a[len(a)-1]
				a[len(a)-1] = nil
				a = a[:len(a)-1]
				break
			}
		}
	}()

	return ch
}

func PublishProblem(event *ProblemEvent) {
	ensureEvents()

	e.lock.Lock()
	defer e.lock.Unlock()

	for _, v := range e.problemSubscribers {
		v.Ch <- event
	}
}
