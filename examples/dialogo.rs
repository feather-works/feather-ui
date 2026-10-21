//! Exemplo do diálogo interno padrão (`feather_ui::dialogo`).
//!
//! `cargo run --example dialogo`: um botão abre o diálogo; o x, o Cancelar,
//! o clique no véu e o Esc fecham.
use feather_ui::dialogo::{self as dlg, Dialogo};
use pelicano::column;
use pelicano::{button, container, text, App, Element, Event, Key, Length, NamedKey, Settings, Style, Task, Theme};

#[derive(Default)]
struct Exemplo {
    aberto: bool,
    aplicado: u32,
}

#[derive(Debug, Clone)]
enum Msg {
    Abrir,
    Fechar,
    Aplicar,
}

impl App for Exemplo {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Abrir => self.aberto = true,
            Msg::Fechar => self.aberto = false,
            Msg::Aplicar => {
                self.aplicado += 1;
                self.aberto = false;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Msg> {
        let base = container(
            column![
                text(format!("Aplicado {} vez(es)", self.aplicado)),
                button(text("Abrir diálogo")).on_press(Msg::Abrir).style(feather_ui::tema::estilo_botao_primario),
            ]
            .spacing(12),
        )
        .padding(24)
        .width(Length::FILL)
        .height(Length::FILL)
        .style(|_| Style::default().background(feather_ui::tema::COR_FUNDO_JANELA()));
        if !self.aberto {
            return base.into();
        }
        let corpo = column![
            dlg::secao("Contagens"),
            dlg::grade_rotulo_valor(
                vec![
                    ("Células preenchidas".into(), "5".into()),
                    ("Vazias".into(), "1".into()),
                    ("Números".into(), "4".into()),
                    ("Valores distintos".into(), "4".into()),
                ],
                2,
            ),
            dlg::secao("Números"),
            dlg::linha_rotulo_valor("Média", "—"),
            dlg::aviso_vazio("Sem números na seleção: média, mínimo e máximo não se aplicam."),
        ]
        .spacing(8);
        Dialogo::new("Estatísticas da seleção", corpo)
            .icone("chart-bar")
            .subtitulo("Intervalo A1:B3 · somente leitura")
            .ao_fechar(Msg::Fechar)
            .topo(80.0)
            .secundario("Cancelar", Some(Msg::Fechar))
            .primario("Aplicar", Some(Msg::Aplicar))
            .sobre(base)
    }

    fn theme(&self) -> Theme {
        feather_ui::tema::tema()
    }

    fn on_event(&self, ev: &Event) -> Option<Msg> {
        match ev {
            Event::Key { key: Key::Named(NamedKey::Escape), pressed: true, .. } => Some(Msg::Fechar),
            _ => None,
        }
    }
}

fn main() -> Result<(), pelicano::Error> {
    pelicano::run(Settings { title: "Diálogo interno".into(), ..Default::default() }, || (Exemplo::default(), Task::none()))
}
