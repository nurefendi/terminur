# Requirements

- Given macOS starts TermiNur, one application row SHALL display Workspace and SSH tabs in the transparent titlebar area, beside rather than on top of the macOS traffic light buttons.
- Native close/minimize/zoom buttons MUST remain accessible and no application tab/button hitbox SHALL cover their reserved area.
- Given the user switches or closes tabs, tab/PTY behavior SHALL match the existing multi-session implementation; clicking New host SHALL still open the form.
- Given a window resize or fullscreen transition, header space SHALL be accounted for in terminal grid sizing without a second empty header row.
- Given a non-macOS build, the transparent titlebar option SHALL NOT be enabled implicitly.
