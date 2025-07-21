package entities

type WateringStatus string //	@Name	WateringStatus

const (
	WateringStatusGood        WateringStatus = "good"
	WateringStatusModerate    WateringStatus = "moderate"
	WateringStatusBad         WateringStatus = "bad"
	WateringStatusJustWatered WateringStatus = "just watered"
	WateringStatusUnknown     WateringStatus = "unknown"
)
