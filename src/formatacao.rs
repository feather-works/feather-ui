//! Controles de formatação da Ribbon, independentes de Slide/Docs/Sheet.
//!
//! Este módulo fornece APENAS estado e visual de popovers, seletores e prévias.
//! A escolha de fonte, tamanho, cor ou borda é sempre uma mensagem do consumidor.
//! Não guarda estilos de células, documentos ou formas e não importa motores.

use pelicano::{button, column, container, row, text, tooltip, Align, Border, Color, Element, Font, Point, Shadow, Space, Style, TooltipPosition};

use crate::{componentes::lista_tipada, icones, seletor_cor, tema};

/// Um único popover aberto por vez. IDs são exclusivos da interface, nunca do documento.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EstadoPopover {
    aberto: Option<String>,
}

impl EstadoPopover {
    pub fn esta_aberto(&self, id: &str) -> bool {
        self.aberto.as_deref() == Some(id)
    }

    /// Repetir a ativação do mesmo controle fecha o popover.
    pub fn alternar(&mut self, id: impl Into<String>) {
        let id = id.into();
        self.aberto = if self.esta_aberto(&id) { None } else { Some(id) };
    }

    pub fn fechar(&mut self) {
        self.aberto = None;
    }

    pub fn aberto(&self) -> Option<&str> {
        self.aberto.as_deref()
    }
}

/// Dica acessível: balão do pelicano e o mesmo texto como nome para o leitor de tela.
pub fn dica<'a, M: Clone + 'a>(
    conteudo: impl Into<Element<'a, M>>,
    descricao: &str,
) -> Element<'a, M> {
    Element::from(tooltip(conteudo, descricao.to_owned(), TooltipPosition::Bottom).padding([4, 8]).style(tema::estilo_dica)).access_label(descricao.to_owned())
}

/// Superfície de popover. Posicionamento e overlay permanecem com o aplicativo
/// para preservar recorte, foco e limites da janela em cada editor.
pub fn popover<'a, M: 'a>(
    titulo: &str,
    conteudo: Element<'a, M>,
) -> Element<'a, M> {
    container(
        column![text(titulo.to_owned()).size(12), conteudo]
            .spacing(8)
            .align(Align::Start),
    )
    .padding(10)
    .style(|_| {
        Style::default()
            .background(tema::COR_PAINEL())
            .text_color(tema::COR_TEXTO())
            .border(Border::rounded(8.0).color(tema::COR_BORDA(), 1.0))
            .shadow(Shadow { color: Color::BLACK.alpha(0.22), offset: Point::new(0.0, 4.0), blur: 12.0 })
    })
    .into()
}

/// Botão com amostra RGBA/sem cor e mensagem tipada do editor.
/// A cor é somente pré-visualização; o seletor HSV/alfa completo permanece em
/// `seletor_cor::painel`. Entradas hex inválidas não são exibidas como preto.
pub fn botao_cor<'a, M: Clone + 'a>(
    rotulo: &str,
    hex: &str,
    mensagem: Option<M>,
    ativo: bool,
) -> Element<'a, M> {
    let valido = seletor_cor::hex_valido(hex);
    let cor = if valido {
        Some(cor_de(hex))
    } else {
        None
    };
    let amostra = container(
        if hex.is_empty() {
            text("×").size(11).color(tema::COR_TEXTO_MUTED())
        } else if !valido {
            text("?").size(11).color(tema::COR_TEXTO_MUTED())
        } else {
            text("").size(11)
        },
    )
    .width(16)
    .height(16)
    .align_x(Align::Center)
    .align_y(Align::Center)
    .style(move |_| {
        let s = Style::default().border(Border::rounded(3.0).color(tema::COR_BORDA(), 1.0));
        match cor {
            Some(c) => s.background(c),
            None => s,
        }
    });
    button(row![amostra, text(rotulo.to_owned()).size(11)]
        .spacing(5).align(Align::Center))
        .padding([4, 7])
        .style(tema::estilo_botao_pequeno(ativo))
        .on_press_maybe(mensagem)
        .into()
}


/// Ícone sobre uma pequena faixa com a cor atual, no padrão da Ribbon do Excel.
/// O controle não aplica estilo: comunica-se com o seletor do aplicativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoCorCompacta {
    /// Balde de tinta: preenchimento da célula.
    Preenchimento,
    /// Letra A: cor do texto.
    Texto,
}

/// Controle estreito sem rótulo ao lado: balde/A em cima, retângulo colorido
/// embaixo. O tooltip preserva o nome acessível e o clique abre o seletor
/// existente, que continua responsável pela escolha da cor.
pub fn botao_cor_compacto<'a, M: Clone + 'a>(
    tipo: TipoCorCompacta,
    hex: &str,
    mensagem: Option<M>,
    ativo: bool,
) -> Element<'a, M> {
    let cor = if seletor_cor::hex_valido(hex) {
        cor_de(hex)
    } else {
        // Sem estilo direto: mostrar o fundo e o texto padrões do tema.
        // Não gravar estas cores como formatação da célula.
        match tipo {
            TipoCorCompacta::Preenchimento => tema::COR_PAINEL(),
            TipoCorCompacta::Texto => tema::COR_TEXTO(),
        }
    };
    let amostra = container(Space::new(0, 4))
        .width(20)
        .height(4)
        .style(move |_| Style::default().background(cor).border(Border::rounded(1.0).color(tema::COR_BORDA(), 0.6)));
    let (icone, descricao): (Element<'a, M>, &str) = match tipo {
        TipoCorCompacta::Preenchimento => (
            icones::icone("bucket", 18.0).into(),
            "Cor de preenchimento da célula",
        ),
        TipoCorCompacta::Texto => (
            text("A").size(18).font(Font::BOLD).into(),
            "Cor do texto",
        ),
    };
    let controle = button(
        column![icone, amostra]
            .spacing(1)
            .align(Align::Center),
    )
    .width(32)
    .padding([2, 5])
    .style(tema::estilo_botao_pequeno(ativo))
    .on_press_maybe(mensagem);
    dica(controle, descricao)
}

/// Dropdown de fontes que preserva tipos e callbacks locais do aplicativo.
pub fn seletor_fonte<'a, M, T>(
    opcoes: &'a [T],
    selecionada: Option<T>,
    escolher: impl Fn(T) -> M + 'a,
    largura: f32,
) -> Element<'a, M>
where
    M: Clone + 'a,
    T: Clone + PartialEq + ToString + 'a,
{
    lista_tipada("Fonte", opcoes, selecionada, escolher, largura)
}

/// Dropdown de tamanho. Não limita a tipografia do motor: cada app define
/// sua escala, seus itens e o tipo da mensagem.
pub fn seletor_tamanho<'a, M, T>(
    opcoes: &'a [T],
    selecionado: Option<T>,
    escolher: impl Fn(T) -> M + 'a,
    largura: f32,
) -> Element<'a, M>
where
    M: Clone + 'a,
    T: Clone + PartialEq + ToString + 'a,
{
    lista_tipada("Tamanho", opcoes, selecionado, escolher, largura)
}

/// Dropdown visual de bordas; nunca aplica mutações em células ou tabelas.
pub fn seletor_bordas<'a, M, T>(
    opcoes: &'a [T],
    selecionada: Option<T>,
    escolher: impl Fn(T) -> M + 'a,
    largura: f32,
) -> Element<'a, M>
where
    M: Clone + 'a,
    T: Clone + PartialEq + ToString + 'a,
{
    lista_tipada("Bordas", opcoes, selecionada, escolher, largura)
}

/// Prévia neutra da tipografia selecionada; não altera objetos do documento.
pub fn previa_estilo<'a, M: 'a>(
    amostra: &str,
    cor_hex: &str,
    tamanho: f32,
    negrito: bool,
) -> Element<'a, M> {
    let cor = if seletor_cor::hex_valido(cor_hex) { cor_de(cor_hex) } else { tema::COR_TEXTO() };
    let fonte = if negrito { Font::BOLD } else { Font::DEFAULT };
    container(
        column![
            text(amostra.to_owned())
                .size(if tamanho.is_finite() { tamanho.clamp(6.0, 96.0) } else { 12.0 })
                .font(fonte)
                .color(cor),
            Space::new(0, 1),
        ]
        .spacing(2),
    )
    .padding([6, 8])
    .style(|_| Style::default().background(tema::COR_PAINEL()).border(Border::rounded(4.0).color(tema::COR_BORDA(), 1.0)))
    .into()
}

fn cor_de(hex: &str) -> Color {
    let [r, g, b, a] = seletor_cor::rgba_de(hex);
    Color::rgb8(r, g, b).alpha(a as f32 / 255.0)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn popover_exclusivo_alterna_e_fecha_sem_comandos_de_documento() {
        let mut estado = EstadoPopover::default();
        assert_eq!(estado.aberto(), None);
        estado.alternar("fonte");
        assert!(estado.esta_aberto("fonte"));
        estado.alternar("cor");
        assert!(estado.esta_aberto("cor"));
        assert!(!estado.esta_aberto("fonte"));
        estado.alternar("cor");
        assert_eq!(estado.aberto(), None);
        estado.alternar("borda");
        estado.fechar();
        assert_eq!(estado.aberto(), None);
    }

    #[test]
    fn controles_aceitam_mensagens_proprias_sem_motor() {
        let _: Element<'_, u8> = botao_cor("Texto", "#33445580", Some(1), true);
        let _: Element<'_, u8> = botao_cor("Fundo", "", None, false);
        let _: Element<'_, u8> = botao_cor("Inválida", "#12xx90", Some(2), false);
        let _: Element<'_, u8> = botao_cor_compacto(TipoCorCompacta::Preenchimento, "#5f6368", Some(1), true);
        let _: Element<'_, u8> = botao_cor_compacto(TipoCorCompacta::Texto, "", Some(2), false);
        let _: Element<'_, u8> = botao_cor_compacto(TipoCorCompacta::Preenchimento, "#ff00ff80", None, false);
        assert!(icones::existe("bucket"), "o balde deve ser um SVG real, não o fallback");
        let _: Element<'_, u8> = seletor_fonte(&["sans", "mono"], Some("sans"), |_| 3, 112.0);
        let _: Element<'_, u8> = seletor_tamanho(&[10u16, 12, 14], Some(12), |_| 4, 64.0);
        let _: Element<'_, u8> = seletor_bordas(&["Todas", "Nenhuma"], None, |_| 5, 91.0);
        let _: Element<'_, u8> = previa_estilo("Exemplo", "#33445580", f32::NAN, true);
        let _: Element<'_, u8> = popover("Cor", text("Testando").into());
        let _: Element<'_, u8> = dica(text("Ajuda"), "Dica");
    }
}
