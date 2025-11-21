package logger

import (
	"context"
	"fmt"
	"io"
	"log"
	"log/slog"
	"path/filepath"
	"runtime"
	"strings"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type LogFormat string

const (
	JSON    LogFormat = "json"
	Text    LogFormat = "text"
	Console LogFormat = "console"
)

type LogLevel string

const (
	Debug LogLevel = "debug"
	Info  LogLevel = "info"
	Warn  LogLevel = "warn"
	Error LogLevel = "error"
)

var _ slog.LogValuer = (*TimeSince)(nil)

type TimeSince struct {
	startTime time.Time
}

func NewTimeSince() *TimeSince {
	return &TimeSince{startTime: time.Now()}
}

func (t *TimeSince) LogValue() slog.Value {
	return slog.StringValue(time.Since(t.startTime).String())
}

func (l LogLevel) ToSLog() slog.Level {
	switch l {
	case Debug:
		return slog.LevelDebug
	case Info:
		return slog.LevelInfo
	case Warn:
		return slog.LevelWarn
	case Error:
		return slog.LevelError
	default:
		return slog.LevelInfo
	}
}

type GroupOrAttrs struct {
	attr  slog.Attr
	group string
}

type PrettyHandlerOptions struct {
	SlogOpts slog.HandlerOptions
}

type PrettyHandler struct {
	opts PrettyHandlerOptions
	slog.Handler
	l   *log.Logger
	goa []GroupOrAttrs
}

func NewPrettyHandler(out io.Writer, opts PrettyHandlerOptions) *PrettyHandler {
	h := &PrettyHandler{
		Handler: slog.NewTextHandler(out, &opts.SlogOpts),
		l:       log.New(out, "", 0),
		opts:    opts,
		goa:     make([]GroupOrAttrs, 0),
	}

	return h
}

func (h *PrettyHandler) WithGroup(group string) slog.Handler {
	lCopy := log.New(h.l.Writer(), h.l.Prefix(), h.l.Flags())
	return &PrettyHandler{
		Handler: h.Handler.WithGroup(group),
		l:       lCopy,
		opts:    h.opts,
		goa:     append(h.goa, GroupOrAttrs{group: group}),
	}
}

func (h *PrettyHandler) WithAttrs(attrs []slog.Attr) slog.Handler {
	newAttrs := make([]GroupOrAttrs, len(attrs))

	for i, attr := range attrs {
		newAttrs[i] = GroupOrAttrs{attr: attr}
	}

	lCopy := log.New(h.l.Writer(), h.l.Prefix(), h.l.Flags())
	return &PrettyHandler{
		Handler: h.Handler.WithAttrs(attrs),
		l:       lCopy,
		opts:    h.opts,
		goa:     append(h.goa, newAttrs...),
	}
}

//nolint:gocritic // ignored because this has to implement slog handler
func (h *PrettyHandler) Handle(_ context.Context, r slog.Record) error {
	level := r.Level.String()
	switch r.Level {
	case slog.LevelDebug:
		level = fmt.Sprintf("\033[35;1m%s\033[0m", level) // Magenta
	case slog.LevelInfo:
		level = fmt.Sprintf("\033[34;1m%s\033[0m", level) // Blue
	case slog.LevelWarn:
		level = fmt.Sprintf("\033[33;1m%s\033[0m", level) // Yellow
	case slog.LevelError:
		level = fmt.Sprintf("\033[31;1m%s\033[0m", level) // Red
	}

	fields := make([]slog.Attr, 0)
	lastGroup := ""
	for _, goa := range h.goa {
		if goa.group != "" {
			lastGroup += goa.group + "."
		} else {
			attr := goa.attr
			attr.Value = attr.Value.Resolve()
			if lastGroup != "" {
				attr.Key = lastGroup + attr.Key
			}

			fields = append(fields, attr)
		}
	}

	r.Attrs(func(a slog.Attr) bool {
		a.Value = a.Value.Resolve()
		if lastGroup != "" {
			a.Key = lastGroup + a.Key
		}
		fields = append(fields, a)
		return true
	})

	logFields := make([]string, 0, len(fields))
	for _, attr := range fields {
		keyColorSeq := "\033[38;5;249;1m" // gray bold
		valueColorSeq := "\033[38;5;249m" // gray

		if attr.Key == "err" || attr.Key == "error" {
			keyColorSeq = "\033[31;1m" // red bold
			valueColorSeq = "\033[31m" // red
		}

		logFields = append(logFields, fmt.Sprintf("%s%s=\033[0m%s\"%v\"\033[0m", keyColorSeq, attr.Key, valueColorSeq, attr.Value.Any()))
	}

	var source string
	if h.opts.SlogOpts.AddSource {
		frame, _ := runtime.CallersFrames([]uintptr{r.PC}).Next()
		dir, file := filepath.Split(frame.File)
		rootDir := utils.RootDir()
		relativeDir, err := filepath.Rel(rootDir, filepath.Dir(dir))
		if err != nil {
			return err
		}

		source = fmt.Sprintf("\033[38;5;98;1;4m%s:%d\033[0m", filepath.Join(relativeDir, file), frame.Line)
	}

	timeStr := fmt.Sprintf("[%s]", r.Time.Format(time.Stamp))
	msg := fmt.Sprintf("\033[36m%s\033[0m", r.Message) // Cyan

	h.l.Println(timeStr, level, source, msg, strings.Join(logFields, " "))

	return nil
}

func CreateLogger(out io.Writer, format LogFormat, level LogLevel) func() *slog.Logger {
	return func() *slog.Logger {
		options := &slog.HandlerOptions{
			AddSource: true,
			Level:     level.ToSLog(),
		}

		var handler slog.Handler
		switch format {
		case JSON:
			handler = slog.NewJSONHandler(out, options)
		case Text:
			handler = slog.NewTextHandler(out, options)
		case Console:
			handler = NewPrettyHandler(out, PrettyHandlerOptions{SlogOpts: *options})
		default:
			handler = slog.NewTextHandler(out, options)
		}

		return slog.New(handler)
	}
}
