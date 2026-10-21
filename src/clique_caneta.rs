//! A ponta da caneta como clique esquerdo no aplicativo inteiro.
//!
//! O winit 0.30 descarta a maior parte dos cliques da caneta (no XWayland,
//! 13 de 15 toques no relato de 19/10/2026), mas a leitura por fora
//! ([`crate::caneta`]) sabe onde a ponta encostou e quando levantou. Este
//! invólucro, posto na raiz da janela, entrega esses toques aos widgets
//! como `MousePressed`/`MouseReleased(Left)` — por dentro do app, sem
//! simular mouse no sistema (o XTest pedia a permissão de área de trabalho
//! remota ao GNOME).
//!
//! Uso: `clique_caneta(conteudo, fonte.as_ref())` em volta do `view` da
//! janela que a leitura acompanha. Os toques chegam pela
//! [`crate::caneta::assinatura_toques`]; ao receber cada lote, o app devolve
//! [`entregar_cliques`] para os widgets verem o toque na hora.
//!
//! Quando a janela recebe o clique da própria caneta, ele vence e o toque
//! não é repetido; enquanto o clique é o do invólucro, o que vier da janela
//! para o botão esquerdo é descartado (não duplica).
use crate::caneta::{FaseToque, FontePressao, ToqueCaneta};
use pelicano::widget::{dispatch, DrawCx, EventCx, LayoutCx, Limits, Node, Widget};
use pelicano::{Captured, Element, Event, Length, MouseButton, Point, Task, WindowId};
use std::time::{Duration, Instant};

/// Clique da janela que chega até este tempo antes do toque é o da caneta.
const JANELA_REAL: Duration = Duration::from_millis(150);

/// Quem está fazendo o clique da ponta encostada agora.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Modo {
    #[default]
    Livre,
    /// O invólucro: o botão esquerdo da janela é descartado até levantar.
    Sintetico,
    /// A própria janela recebeu o aperto da caneta.
    Real,
}

#[derive(Default)]
struct Estado {
    modo: Modo,
    real_em: Option<Instant>,
    real_soltou: bool,
}

impl Estado {
    /// Converte um toque da ponta em eventos de mouse.
    fn traduzir(&mut self, toque: &ToqueCaneta, agora: Instant) -> Vec<Event> {
        let mover = Event::CursorMoved(toque.posicao);
        let esquerdo = MouseButton::Left;
        match toque.fase {
            FaseToque::Encostou => {
                if self.real_em.is_some_and(|t| agora.saturating_duration_since(t) <= JANELA_REAL) {
                    self.modo = Modo::Real;
                    return Vec::new();
                }
                self.modo = Modo::Sintetico;
                vec![mover, Event::MousePressed(esquerdo)]
            }
            FaseToque::Moveu if self.modo == Modo::Sintetico => vec![mover],
            FaseToque::Moveu => Vec::new(),
            FaseToque::Levantou => {
                let eventos = match self.modo {
                    Modo::Sintetico => vec![mover, Event::MouseReleased(esquerdo)],
                    // A janela apertou e não soltou: o botão só age na soltura.
                    Modo::Real if !self.real_soltou => vec![Event::MouseReleased(esquerdo)],
                    _ => Vec::new(),
                };
                self.modo = Modo::Livre;
                self.real_em = None;
                eventos
            }
        }
    }

    /// Um evento da janela passa? Durante o clique do invólucro, o botão
    /// esquerdo real é descartado.
    fn real_passa(&self, evento: &Event) -> bool {
        !(self.modo == Modo::Sintetico && e_esquerdo(evento))
    }

    /// Anota o botão esquerdo real que passou.
    fn anotar_real(&mut self, evento: &Event, agora: Instant) {
        match evento {
            Event::MousePressed(MouseButton::Left) => {
                self.real_em = Some(agora);
                self.real_soltou = false;
            }
            Event::MouseReleased(MouseButton::Left) => self.real_soltou = true,
            _ => {}
        }
    }
}

fn e_esquerdo(evento: &Event) -> bool {
    matches!(evento, Event::MousePressed(MouseButton::Left) | Event::MouseReleased(MouseButton::Left))
}

/// Ver o módulo.
pub struct CliqueCaneta<'a, M> {
    conteudo: Element<'a, M>,
    fonte: Option<FontePressao>,
}

/// Embrulha a raiz da janela. Sem `fonte` (leitura desligada), só repassa.
pub fn clique_caneta<'a, M: 'a>(conteudo: impl Into<Element<'a, M>>, fonte: Option<&FontePressao>) -> CliqueCaneta<'a, M> {
    CliqueCaneta { conteudo: conteudo.into(), fonte: fonte.cloned() }
}

/// Faz os toques do `lote` (o da [`crate::caneta::assinatura_toques`]) chegarem
/// aos widgets já: sem ela, o clique da caneta espera o próximo evento da janela.
/// Passa à janela um `CursorMoved` onde a ponta está (lá o ponteiro também está).
pub fn entregar_cliques<M: Send + 'static>(janela: WindowId, lote: &[ToqueCaneta]) -> Task<M> {
    match lote.iter().rev().find(|t| !t.na_tinta) {
        Some(t) => Task::window_event(janela, Event::CursorMoved(t.posicao)),
        None => Task::none(),
    }
}

impl<M> Widget<M> for CliqueCaneta<'_, M> {
    fn size(&self) -> (Length, Length) {
        self.conteudo.widget().size()
    }
    fn layout(&self, cx: &mut LayoutCx, l: Limits) -> Node {
        let n = cx.child(0, &self.conteudo, l);
        Node { rect: n.rect, key: cx.key(), children: vec![n], ..Default::default() }
    }
    fn draw(&self, n: &Node, cx: &mut DrawCx) {
        self.conteudo.widget().draw(&n.children[0], cx);
    }
    fn opaque_at(&self, n: &Node, p: Point) -> bool {
        self.conteudo.widget().opaque_at(&n.children[0], p)
    }
    fn event(&self, n: &Node, ev: &Event, cx: &mut EventCx<M>) -> Captured {
        let filho = &n.children[0];
        let Some(fonte) = &self.fonte else { return self.conteudo.widget().event(filho, ev, cx) };
        let mut estado = std::mem::take(cx.states.get_mut::<Estado>(n.key));
        let agora = Instant::now();
        // O real antes dos toques: um aperto da janela que chega junto do encostar vence.
        let mut pegou = if estado.real_passa(ev) {
            estado.anotar_real(ev, agora);
            self.conteudo.widget().event(filho, ev, cx)
        } else {
            Captured::Yes
        };
        let real = cx.cursor;
        for toque in fonte.recolher_cliques() {
            for sintetico in estado.traduzir(&toque, agora) {
                // `dispatch` também decide menu aberto por cima e tira o foco no aperto.
                cx.cursor = Some(toque.posicao);
                cx.redraw = true;
                if dispatch(&self.conteudo, filho, &sintetico, cx) == Captured::Yes {
                    pegou = Captured::Yes;
                }
            }
        }
        cx.cursor = real;
        *cx.states.get_mut::<Estado>(n.key) = estado;
        pegou
    }
}

impl<'a, M: 'a> From<CliqueCaneta<'a, M>> for Element<'a, M> {
    fn from(c: CliqueCaneta<'a, M>) -> Self {
        Element::new(c)
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use pelicano::widget::{layout, MemoryClipboard};
    use pelicano::{button, Color, Renderer, Size, States, Theme};

    fn toque(fase: FaseToque, x: f32) -> ToqueCaneta {
        ToqueCaneta { fase, posicao: Point::new(x, 10.0), pressao: Some(0.5), instante: Instant::now(), na_tinta: false, borracha: false }
    }

    fn apertos(eventos: &[Event]) -> Vec<&Event> {
        eventos.iter().filter(|e| !matches!(e, Event::CursorMoved(_))).collect()
    }

    #[test]
    fn toque_sem_clique_da_janela_vira_aperto_arrasto_e_soltura() {
        let mut e = Estado::default();
        let t0 = Instant::now();
        let a = e.traduzir(&toque(FaseToque::Encostou, 5.0), t0);
        assert_eq!(apertos(&a), [&Event::MousePressed(MouseButton::Left)]);
        assert_eq!(e.traduzir(&toque(FaseToque::Moveu, 9.0), t0), [Event::CursorMoved(Point::new(9.0, 10.0))]);
        // Enquanto o invólucro aperta, o esquerdo da janela não passa (não duplica).
        assert!(!e.real_passa(&Event::MousePressed(MouseButton::Left)));
        assert!(e.real_passa(&Event::MousePressed(MouseButton::Right)));
        let s = e.traduzir(&toque(FaseToque::Levantou, 9.0), t0);
        assert_eq!(apertos(&s), [&Event::MouseReleased(MouseButton::Left)]);
        assert!(e.real_passa(&Event::MousePressed(MouseButton::Left)), "mouse volta a passar");
    }

    #[test]
    fn clique_da_janela_que_veio_antes_vence_e_soltura_que_faltou_e_completada() {
        let pressionar = Event::MousePressed(MouseButton::Left);
        let soltar = Event::MouseReleased(MouseButton::Left);
        let t0 = Instant::now();
        // Aperto e soltura da janela chegaram: o toque não repete nada.
        let mut e = Estado::default();
        e.anotar_real(&pressionar, t0);
        e.anotar_real(&soltar, t0);
        assert!(e.traduzir(&toque(FaseToque::Encostou, 5.0), t0).is_empty());
        assert!(e.traduzir(&toque(FaseToque::Levantou, 5.0), t0).is_empty());
        // Só o aperto chegou: a soltura sai do invólucro.
        let mut e = Estado::default();
        e.anotar_real(&pressionar, t0);
        assert!(e.traduzir(&toque(FaseToque::Encostou, 5.0), t0).is_empty());
        assert_eq!(e.traduzir(&toque(FaseToque::Levantou, 5.0), t0), [Event::MouseReleased(MouseButton::Left)]);
        // Clique do mouse antigo não conta como o da caneta.
        let mut e = Estado::default();
        e.anotar_real(&pressionar, t0);
        let depois = t0 + JANELA_REAL + Duration::from_millis(1);
        assert_eq!(apertos(&e.traduzir(&toque(FaseToque::Encostou, 5.0), depois)).len(), 1);
    }

    #[test]
    fn toque_da_caneta_aperta_o_botao_pelo_involucro() {
        let fonte = FontePressao::nova();
        let raiz: Element<'_, u8> = clique_caneta(button("ok").on_press(7u8), Some(&fonte)).into();
        let (mut r, mut st, mut area) = (Renderer::new(), States::default(), MemoryClipboard::default());
        let tela = Size::new(200.0, 100.0);
        r.begin(tela, 1.0, Color::TRANSPARENT);
        let n = layout(&raiz, tela, &mut r, &st, &Theme::dark());
        let mut evento = |st: &mut States, r: &mut Renderer, ev: Event, cursor: Option<Point>| {
            let mut msgs = Vec::new();
            let mut cx = EventCx { renderer: r, states: st, cursor, messages: &mut msgs, window: &mut Vec::new(), redraw: false, clipboard: &mut area };
            let c = dispatch(&raiz, &n, &ev, &mut cx);
            (msgs, c)
        };
        fonte.enviar_toque(toque(FaseToque::Encostou, 10.0));
        fonte.enviar_toque(toque(FaseToque::Levantou, 10.0));
        let lote = fonte.recolher_toques();
        assert_eq!(lote.len(), 2);
        let _: Task<u8> = entregar_cliques(WindowId::MAIN, &lote);
        // O cursor real está longe: o clique é o do toque, em (10, 10).
        let (msgs, c) = evento(&mut st, &mut r, Event::CursorMoved(Point::new(150.0, 90.0)), Some(Point::new(150.0, 90.0)));
        assert_eq!((msgs, c), (vec![7], Captured::Yes));
        // Sem toque pendente, o mouse segue normal.
        let (msgs, _) = evento(&mut st, &mut r, Event::MousePressed(MouseButton::Left), Some(Point::new(10.0, 10.0)));
        assert!(msgs.is_empty());
        let (msgs, _) = evento(&mut st, &mut r, Event::MouseReleased(MouseButton::Left), Some(Point::new(10.0, 10.0)));
        assert_eq!(msgs, vec![7]);
    }
}
