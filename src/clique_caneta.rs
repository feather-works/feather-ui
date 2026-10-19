//! A ponta da caneta como clique esquerdo no aplicativo inteiro.
//!
//! O winit 0.30 descarta a maior parte dos cliques da caneta (no XWayland,
//! 13 de 15 toques no relato de 19/10/2026), mas a leitura por fora
//! ([`crate::caneta`]) sabe onde a ponta encostou e quando levantou. Este
//! invólucro, posto na raiz da janela, entrega esses toques aos widgets
//! como `ButtonPressed`/`ButtonReleased(Left)` — por dentro do app, sem
//! simular mouse no sistema (o XTest pedia a permissão de área de trabalho
//! remota ao GNOME).
//!
//! Uso: `clique_caneta(conteudo, fonte.as_ref())` em volta do `view` da
//! janela que a leitura acompanha. Os toques chegam acordando o app pela
//! [`crate::caneta::assinatura_toques`]; o app precisa ouvi-la.
//!
//! Quando o Iced recebe o clique da própria caneta, ele vence e o toque
//! não é repetido; enquanto o clique é o do invólucro, o que vier do Iced
//! para o botão esquerdo é descartado (não duplica).
use crate::caneta::{FaseToque, FontePressao, ToqueCaneta};
use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::{event, mouse, Element, Event, Length, Point, Rectangle, Size, Vector};
use std::time::{Duration, Instant};

/// Clique do Iced que chega até este tempo antes do toque é o da caneta.
const JANELA_REAL: Duration = Duration::from_millis(150);

/// Quem está fazendo o clique da ponta encostada agora.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Modo {
    #[default]
    Livre,
    /// O invólucro: o botão esquerdo do Iced é descartado até levantar.
    Sintetico,
    /// O próprio Iced recebeu o aperto da caneta.
    Real,
}

#[derive(Default)]
struct Estado {
    modo: Modo,
    real_em: Option<Instant>,
    real_soltou: bool,
    /// Eventos do ciclo atual: entregues ao overlay (se houver) e depois à
    /// base. `None` no cursor = o overlay capturou.
    lote: Vec<(mouse::Event, Option<mouse::Cursor>)>,
}

impl Estado {
    /// Converte um toque da ponta em eventos de mouse.
    fn traduzir(&mut self, toque: &ToqueCaneta, agora: Instant) -> Vec<mouse::Event> {
        let mover = mouse::Event::CursorMoved { position: toque.posicao };
        let esquerdo = mouse::Button::Left;
        match toque.fase {
            FaseToque::Encostou => {
                if self.real_em.is_some_and(|t| agora.saturating_duration_since(t) <= JANELA_REAL) {
                    self.modo = Modo::Real;
                    return Vec::new();
                }
                self.modo = Modo::Sintetico;
                vec![mover, mouse::Event::ButtonPressed(esquerdo)]
            }
            FaseToque::Moveu if self.modo == Modo::Sintetico => vec![mover],
            FaseToque::Moveu => Vec::new(),
            FaseToque::Levantou => {
                let eventos = match self.modo {
                    Modo::Sintetico => vec![mover, mouse::Event::ButtonReleased(esquerdo)],
                    // O Iced apertou e não soltou: o botão só age na soltura.
                    Modo::Real if !self.real_soltou => vec![mouse::Event::ButtonReleased(esquerdo)],
                    _ => Vec::new(),
                };
                self.modo = Modo::Livre;
                self.real_em = None;
                eventos
            }
        }
    }

    /// Um evento do Iced passa? Durante o clique do invólucro, o botão
    /// esquerdo real é descartado.
    fn real_passa(&self, evento: &Event) -> bool {
        !(self.modo == Modo::Sintetico && e_esquerdo(evento))
    }

    /// Anota o botão esquerdo real que passou (só a base chama: uma vez por evento).
    fn anotar_real(&mut self, evento: &Event, agora: Instant) {
        match evento {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                self.real_em = Some(agora);
                self.real_soltou = false;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => self.real_soltou = true,
            _ => {}
        }
    }

    fn recolher(&mut self, fonte: &FontePressao, cursor: impl Fn(Point) -> Option<mouse::Cursor>) {
        let agora = Instant::now();
        for toque in fonte.recolher_cliques() {
            for evento in self.traduzir(&toque, agora) {
                self.lote.push((evento, cursor(toque.posicao)));
            }
        }
    }
}

fn e_esquerdo(evento: &Event) -> bool {
    matches!(
        evento,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left) | mouse::Event::ButtonReleased(mouse::Button::Left))
    )
}

/// Ver o módulo.
pub struct CliqueCaneta<'a, Message, Theme, Renderer> {
    conteudo: Element<'a, Message, Theme, Renderer>,
    fonte: Option<FontePressao>,
}

/// Embrulha a raiz da janela. Sem `fonte` (leitura desligada), só repassa.
pub fn clique_caneta<'a, Message, Theme, Renderer>(
    conteudo: impl Into<Element<'a, Message, Theme, Renderer>>,
    fonte: Option<&FontePressao>,
) -> CliqueCaneta<'a, Message, Theme, Renderer> {
    CliqueCaneta { conteudo: conteudo.into(), fonte: fonte.cloned() }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for CliqueCaneta<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        self.conteudo.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.conteudo.as_widget().size_hint()
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.conteudo.as_widget().layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.conteudo.as_widget().draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Estado>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Estado::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.conteudo)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.conteudo));
    }

    fn operate(&self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.conteudo.as_widget().operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        evento: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let Tree { state, children, .. } = tree;
        let estado = state.downcast_mut::<Estado>();
        let filho = &mut children[0];
        let mut status = event::Status::Ignored;
        if let Some(fonte) = &self.fonte {
            // Sem overlay aberto, o lote do ciclo ainda não saiu da fonte.
            if estado.lote.is_empty() {
                estado.recolher(fonte, |p| Some(mouse::Cursor::Available(p)));
            }
            for (sintetico, cursor_sintetico) in std::mem::take(&mut estado.lote) {
                let Some(c) = cursor_sintetico else { continue };
                status = status.merge(self.conteudo.as_widget_mut().on_event(
                    filho, Event::Mouse(sintetico), layout, c, renderer, clipboard, shell, viewport,
                ));
            }
            if !estado.real_passa(&evento) {
                return event::Status::Captured;
            }
            estado.anotar_real(&evento, Instant::now());
        }
        status.merge(self.conteudo.as_widget_mut().on_event(
            filho, evento, layout, cursor, renderer, clipboard, shell, viewport,
        ))
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.conteudo.as_widget().mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let Tree { state, children, .. } = tree;
        let interno = self.conteudo.as_widget_mut().overlay(&mut children[0], layout, renderer, translation)?;
        match &self.fonte {
            Some(fonte) => Some(overlay::Element::new(Box::new(Sobreposicao {
                interno,
                estado: state.downcast_mut::<Estado>(),
                fonte: fonte.clone(),
            }))),
            None => Some(interno),
        }
    }
}

/// O overlay aberto (lista de um `pick_list`, por exemplo) recebe os
/// eventos antes da base: o lote do ciclo sai da fonte aqui e o que ele
/// captura não desce. shortcut: só o overlay de primeiro nível recebe o
/// toque (um overlay aninhado nele, não); trocar se algum app abrir
/// submenu como overlay do Iced.
struct Sobreposicao<'a, 'b, Message, Theme, Renderer> {
    interno: overlay::Element<'a, Message, Theme, Renderer>,
    estado: &'b mut Estado,
    fonte: FontePressao,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer> for Sobreposicao<'_, '_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        self.interno.layout(renderer, bounds)
    }

    fn draw(&self, renderer: &mut Renderer, theme: &Theme, style: &renderer::Style, layout: Layout<'_>, cursor: mouse::Cursor) {
        self.interno.draw(renderer, theme, style, layout, cursor);
    }

    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.interno.operate(layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        evento: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let mut status = event::Status::Ignored;
        if self.estado.lote.is_empty() {
            self.estado.recolher(&self.fonte, |p| Some(mouse::Cursor::Available(p)));
            for (sintetico, c) in &mut self.estado.lote {
                let Some(mouse::Cursor::Available(p)) = *c else { continue };
                let s = self.interno.on_event(Event::Mouse(*sintetico), layout, mouse::Cursor::Available(p), renderer, clipboard, shell);
                status = status.merge(s);
                // Capturado não desce; sobre o overlay, a base não vê o cursor.
                *c = match s {
                    event::Status::Captured => None,
                    _ if self.interno.is_over(layout, renderer, p) => Some(mouse::Cursor::Unavailable),
                    _ => Some(mouse::Cursor::Available(p)),
                };
            }
        }
        if !self.estado.real_passa(&evento) {
            return event::Status::Captured;
        }
        status.merge(self.interno.on_event(evento, layout, cursor, renderer, clipboard, shell))
    }

    fn mouse_interaction(&self, layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle, renderer: &Renderer) -> mouse::Interaction {
        self.interno.mouse_interaction(layout, cursor, viewport, renderer)
    }

    fn is_over(&self, layout: Layout<'_>, renderer: &Renderer, cursor_position: Point) -> bool {
        self.interno.is_over(layout, renderer, cursor_position)
    }

    fn overlay<'c>(&'c mut self, layout: Layout<'_>, renderer: &Renderer) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        self.interno.overlay(layout, renderer)
    }
}

impl<'a, Message, Theme, Renderer> From<CliqueCaneta<'a, Message, Theme, Renderer>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(c: CliqueCaneta<'a, Message, Theme, Renderer>) -> Self {
        Element::new(c)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn toque(fase: FaseToque, x: f32) -> ToqueCaneta {
        ToqueCaneta { fase, posicao: Point::new(x, 10.0), pressao: Some(0.5), instante: Instant::now(), na_tinta: false, borracha: false }
    }

    fn apertos(eventos: &[mouse::Event]) -> Vec<&mouse::Event> {
        eventos.iter().filter(|e| !matches!(e, mouse::Event::CursorMoved { .. })).collect()
    }

    #[test]
    fn toque_sem_clique_do_iced_vira_aperto_arrasto_e_soltura() {
        let mut e = Estado::default();
        let t0 = Instant::now();
        let a = e.traduzir(&toque(FaseToque::Encostou, 5.0), t0);
        assert_eq!(apertos(&a), [&mouse::Event::ButtonPressed(mouse::Button::Left)]);
        assert_eq!(e.traduzir(&toque(FaseToque::Moveu, 9.0), t0), [mouse::Event::CursorMoved { position: Point::new(9.0, 10.0) }]);
        // Enquanto o invólucro aperta, o esquerdo do Iced não passa (não duplica).
        assert!(!e.real_passa(&Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))));
        assert!(e.real_passa(&Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))));
        let s = e.traduzir(&toque(FaseToque::Levantou, 9.0), t0);
        assert_eq!(apertos(&s), [&mouse::Event::ButtonReleased(mouse::Button::Left)]);
        assert!(e.real_passa(&Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))), "mouse volta a passar");
    }

    #[test]
    fn clique_do_iced_que_veio_antes_vence_e_soltura_que_faltou_e_completada() {
        let pressionar = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        let soltar = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
        let t0 = Instant::now();
        // Aperto e soltura do Iced chegaram: o toque não repete nada.
        let mut e = Estado::default();
        e.anotar_real(&pressionar, t0);
        e.anotar_real(&soltar, t0);
        assert!(e.traduzir(&toque(FaseToque::Encostou, 5.0), t0).is_empty());
        assert!(e.traduzir(&toque(FaseToque::Levantou, 5.0), t0).is_empty());
        // Só o aperto chegou: a soltura sai do invólucro.
        let mut e = Estado::default();
        e.anotar_real(&pressionar, t0);
        assert!(e.traduzir(&toque(FaseToque::Encostou, 5.0), t0).is_empty());
        assert_eq!(e.traduzir(&toque(FaseToque::Levantou, 5.0), t0), [mouse::Event::ButtonReleased(mouse::Button::Left)]);
        // Clique do mouse antigo não conta como o da caneta.
        let mut e = Estado::default();
        e.anotar_real(&pressionar, t0);
        let depois = t0 + JANELA_REAL + Duration::from_millis(1);
        assert_eq!(apertos(&e.traduzir(&toque(FaseToque::Encostou, 5.0), depois)).len(), 1);
    }
}
