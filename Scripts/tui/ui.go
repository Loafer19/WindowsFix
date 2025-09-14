package main

import (
	"fmt"

	tea "github.com/charmbracelet/bubbletea"
)

type model struct {
	cursor   int
	choices  []string
	selected map[int]struct{}
	status   string
	running  bool
}

func initialModel() model {
	return model{
		choices:  []string{"Unpin Network Folder", "Set Grouping to None", "Unpin Quick Access Folders", "Exit"},
		selected: make(map[int]struct{}),
		status:   "Select an option:",
		running:  false,
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
			switch m.cursor {
			case 0: // Unpin Network Folder
				m.running = true
				m.status = "Unpinning network folder..."
				return m, unpinNetwork()
			case 1: // Set Grouping to None
				m.running = true
				m.status = "Setting folder grouping to none..."
				return m, setGroupingNone()
			case 2: // Unpin Quick Access Folders
				m.running = true
				m.status = "Unpinning Quick Access folders..."
				return m, unpinQuickAccess()
			case 3: // Exit
				return m, tea.Quit
			}
		}
	case statusMsg:
		m.status = string(msg)
		m.running = false
	}

	return m, nil
}

func (m model) View() string {
	s := "Scripts TUI\n\n"

	for i, choice := range m.choices {
		cursor := " "
		if m.cursor == i {
			cursor = ">"
		}

		checked := " "
		if _, ok := m.selected[i]; ok {
			checked = "x"
		}

		s += fmt.Sprintf("%s [%s] %s\n", cursor, checked, choice)
	}

	s += "\n" + m.status + "\n\n"
	s += "Use arrow keys to navigate, Enter to select, q to quit."

	return s
}
