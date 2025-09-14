package main

import (
	"fmt"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
	lips "github.com/charmbracelet/lipgloss"
)

type model struct {
	cursor    int
	choices   []string
	selected  map[int]struct{}
	status    string
	running   bool
	actions   []menuAction
	completed []bool
	lastRun   int
}

type menuAction struct {
	action      func() tea.Cmd
	status      string
	setsRunning bool
}

func initialModel() model {
	return model{
		choices:  []string{"Explorer: Unpin Network Folder", "Explorer: Globally Set Grouping To None", "Explorer: Unpin Quick Access Folders", "Desktop: Remove All Icons", "Exit"},
		selected: make(map[int]struct{}),
		status:   "Choose an option and press Enter to execute ;)",
		running:  false,
		actions: []menuAction{
			{unpinNetwork, "Unpinning network folder...", true},
			{setGroupingNone, "Globally setting grouping to none...", true},
			{unpinQuickAccess, "Unpinning Quick Access folders...", true},
			{removeAllIcons, "Hiding all desktop icons...", true},
			{func() tea.Cmd { return tea.Quit }, "", false},
		},
		completed: make([]bool, 5),
		lastRun:   -1,
	}
}

func (m model) Init() tea.Cmd {
	return nil
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			return m, tea.Quit
		case "up", "k":
			if m.cursor > 0 {
				m.cursor--
			}
		case "down", "j":
			if m.cursor < len(m.choices)-1 {
				m.cursor++
			}
		case "enter", " ":
			if m.cursor >= 0 && m.cursor < len(m.actions) && !m.completed[m.cursor] {
				act := m.actions[m.cursor]
				if act.setsRunning {
					m.running = true
					m.lastRun = m.cursor
				}
				if act.status != "" {
					m.status = act.status
				}
				return m, act.action()
			}
		}
	case statusMsg:
		m.status = string(msg)
		m.running = false
		if m.lastRun >= 0 {
			m.completed[m.lastRun] = true
		}
	}

	return m, nil
}

func (m model) View() string {
	var s strings.Builder

	// Styles using lips
	titleStyle := lips.NewStyle().
		Bold(true).
		Foreground(lips.Color("39")).
		MarginTop(1).
		MarginBottom(1)

	groupStyle := lips.NewStyle().
		MarginTop(1).
		MarginBottom(1)

	cursorStyle := lips.NewStyle().
		Foreground(lips.Color("11"))

	checkedStyle := lips.NewStyle().
		Foreground(lips.Color("2"))

	statusStyle := lips.NewStyle().
		Foreground(lips.Color("244")).
		MarginTop(1).
		MarginBottom(1)

	successStyle := lips.NewStyle().
		Foreground(lips.Color("2")).
		Bold(true).
		MarginTop(1).
		MarginBottom(1)

	errorStyle := lips.NewStyle().
		Foreground(lips.Color("1")).
		Bold(true).
		MarginTop(1).
		MarginBottom(1)

	helpStyle := lips.NewStyle().
		Foreground(lips.Color("240"))

	s.WriteString(titleStyle.Render("WindowsFix - Scripts TUI"))
	s.WriteString(groupStyle.Render("Available Options:"))
	s.WriteString("\n")

	for i, choice := range m.choices {
		cursor := " "
		if m.cursor == i {
			cursor = cursorStyle.Render(">")
		}

		checked := " "
		if m.completed[i] {
			checked = checkedStyle.Render("✓")
		}

		line := fmt.Sprintf("%s [%s] %s", cursor, checked, choice)
		s.WriteString(line)
		s.WriteString("\n")
	}

	if strings.HasPrefix(m.status, "Success:") {
		s.WriteString(successStyle.Render(m.status))
	} else if strings.HasPrefix(m.status, "Error:") {
		s.WriteString(errorStyle.Render(m.status))
	} else {
		s.WriteString(statusStyle.Render(m.status))
	}
	s.WriteString("\n")
	s.WriteString(helpStyle.Render("Use arrow keys to navigate, q to quit."))

	return s.String()
}
