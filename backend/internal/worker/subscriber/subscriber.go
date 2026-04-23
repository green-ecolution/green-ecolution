package subscriber

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

type UpdateTreeSubscriber struct {
	tcs ports.TreeClusterService
}

func NewUpdateTreeSubscriber(tcs ports.TreeClusterService) *UpdateTreeSubscriber {
	return &UpdateTreeSubscriber{
		tcs: tcs,
	}
}

func (s *UpdateTreeSubscriber) EventType() shared.EventType {
	return shared.EventTypeUpdateTree
}

func (s *UpdateTreeSubscriber) HandleEvent(ctx context.Context, e shared.Event) error {
	event := e.(shared.EventUpdateTree)
	return s.tcs.HandleUpdateTree(ctx, &event)
}

type CreateTreeSubscriber struct {
	tcs ports.TreeClusterService
}

func NewCreateTreeSubscriber(tcs ports.TreeClusterService) *CreateTreeSubscriber {
	return &CreateTreeSubscriber{
		tcs: tcs,
	}
}

func (s *CreateTreeSubscriber) EventType() shared.EventType {
	return shared.EventTypeCreateTree
}

func (s *CreateTreeSubscriber) HandleEvent(ctx context.Context, e shared.Event) error {
	event := e.(shared.EventCreateTree)
	return s.tcs.HandleCreateTree(ctx, &event)
}

type DeleteTreeSubscriber struct {
	tcs ports.TreeClusterService
}

func NewDeleteTreeSubscriber(tcs ports.TreeClusterService) *DeleteTreeSubscriber {
	return &DeleteTreeSubscriber{
		tcs: tcs,
	}
}

func (s *DeleteTreeSubscriber) EventType() shared.EventType {
	return shared.EventTypeDeleteTree
}

func (s *DeleteTreeSubscriber) HandleEvent(ctx context.Context, e shared.Event) error {
	event := e.(shared.EventDeleteTree)
	return s.tcs.HandleDeleteTree(ctx, &event)
}

type CreateSensorDataSubscriber struct {
	tcSvc   ports.TreeClusterService
	treeSvc ports.TreeService
}

func NewSensorDataSubscriber(tcSvc ports.TreeClusterService, treeSvc ports.TreeService) *CreateSensorDataSubscriber {
	return &CreateSensorDataSubscriber{
		tcSvc:   tcSvc,
		treeSvc: treeSvc,
	}
}

func (s *CreateSensorDataSubscriber) EventType() shared.EventType {
	return shared.EventTypeNewSensorData
}

func (s *CreateSensorDataSubscriber) HandleEvent(ctx context.Context, e shared.Event) error {
	event := e.(shared.EventNewSensorData)
	if err := s.treeSvc.HandleNewSensorData(ctx, &event); err != nil {
		return err
	}

	return s.tcSvc.HandleNewSensorData(ctx, &event)
}

type UpdateWateringPlanSubscriber struct {
	tcSvc ports.TreeClusterService
}

func NewUpdateWateringPlanSubscriber(tcSvc ports.TreeClusterService) *UpdateWateringPlanSubscriber {
	return &UpdateWateringPlanSubscriber{
		tcSvc: tcSvc,
	}
}

func (s *UpdateWateringPlanSubscriber) EventType() shared.EventType {
	return shared.EventTypeUpdateWateringPlan
}

func (s *UpdateWateringPlanSubscriber) HandleEvent(ctx context.Context, e shared.Event) error {
	event := e.(shared.EventUpdateWateringPlan)
	if err := s.tcSvc.HandleUpdateWateringPlan(ctx, &event); err != nil {
		return err
	}

	return s.tcSvc.HandleUpdateWateringPlan(ctx, &event)
}
