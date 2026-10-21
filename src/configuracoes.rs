//! Estrutura visual da área de configurações do FeatherWorks.
//! Não conhece documentos nem preferências: recebe apenas eventos do app.
use pelicano::{button, column, container, horizontal_rule, horizontal_space, row, text, Align, Border, Element, Length, Space, Style};
use crate::{icones, tema::*};

pub fn linha<'a, M: 'a>(
    titulo: &str,
    dica: &str,
    controle: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    column![
        row![
            column![
                text(titulo.to_owned()).size(13.5).color(COR_TEXTO()),
                text(dica.to_owned()).size(11.5).color(COR_TEXTO_MUTED()),
            ].spacing(3).width(Length::FILL),
            controle.into(),
        ].spacing(14).align(Align::Center).padding([14u16, 0]),
        horizontal_rule(1.0).color(COR_BORDA_SUAVE()),
    ].spacing(0).width(Length::FILL).into()
}

pub fn menu<'a, M: Clone + 'a>(
    titulo: &str,
    itens: impl IntoIterator<Item = (String, bool, M)>,
) -> Element<'a, M> {
    let mut col = column![
        text(titulo.to_owned()).size(16).color(COR_TEXTO()),
        Space::new(0.0, 10.0),
    ].spacing(4);
    for (nome, ativo, msg) in itens {
        let simbolo = match nome.as_str() {
            "Aparência" => "palette",
            "Editor" => "file-code",
            "Atalhos" => "keyboard",
            "Sobre" => "info-circle",
            _ => "settings",
        };
        col = col.push(
            // A linha enche o botão: o botão do pelicano centraliza o que é menor que ele.
            button(row![icones::icone(simbolo, 16.0), text(nome).size(13)]
                .spacing(10).align(Align::Center).width(Length::FILL))
                .width(Length::FILL)
                .padding([9u16, 11])
                .style(estilo_botao_pequeno(ativo))
                .on_press(msg),
        );
    }
    container(col.padding([20u16, 12])).width(190.0).height(Length::FILL)
        .style(|_| Style::default()
            .background(COR_FUNDO_JANELA())
            .border(Border::default().color(COR_BORDA_SUAVE(), 1.0)))
        .into()
}

pub fn tela<'a, M: Clone + 'a>(
    menu: Element<'a, M>,
    titulo: &str,
    corpo: Element<'a, M>,
    fechar: Option<(&str, M)>,
) -> Element<'a, M> {
    let mut cabecalho = row![
        text(titulo.to_owned()).size(20).color(COR_TEXTO()),
        horizontal_space(),
    ].align(Align::Center);
    if let Some((_nome, msg)) = fechar {
        cabecalho = cabecalho.push(
            button(icones::icone("x", 15.0)).padding([5u16, 7])
                .style(estilo_botao_pequeno(false)).on_press(msg)
        );
    }
    row![
        menu,
        // Até 1100 de largura, centrado: em janela grande não sobra metade da tela vazia.
        rolagem(container(
            container(column![cabecalho, corpo].spacing(12)).max_width(1100.0).width(Length::FILL)
        ).padding([20u16, 26]).width(Length::FILL).align_x(Align::Center)),
    ].height(Length::FILL).into()
}
