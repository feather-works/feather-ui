//! Ajuda FeatherWorks: landing de tópicos + navegação em três grupos.
//! O modelo, a busca e os eventos continuam compartilhados entre os apps;
//! todas as cores vêm dos tokens da preferência ativa (claro/escuro/acento).
use super::*;
use crate::{consulta, icones, tema};
use crate::consulta::estilo_cartao as estilo_card;
use pelicano::{button, column, container, horizontal_space, row, scrollable, text, text_input, Align, Border, Element, Font, Length, Style, Theme};

#[derive(Clone)]
struct Guia {
    titulo: String,
    descricao: String,
    icone: &'static str,
    destino: Evento,
}
fn texto_negrito(nome: impl Into<String>, tamanho: f32) -> Element<'static, Evento> {
    consulta::negrito(nome, tamanho)
}

fn barra(_: &Theme) -> Style {
    Style::default().background(tema::COR_PAINEL()).border(Border::rounded(12.0).color(tema::COR_BORDA_SUAVE(), 1.))
}

fn icone_hero() -> Element<'static, Evento> {
    consulta::marca()
}
fn ilustra_planilha() -> Element<'static, Evento> {
    consulta::ilustracao_planilha()
}
fn resumo(t: &Topico) -> String {
    t.linhas.iter().find_map(|l| match l {
        Linha::Texto(txt) | Linha::Marcador(txt) => Some(txt.clone()),
        _ => None,
    }).map(|txt| {
        let mut s=txt.chars().take(84).collect::<String>();
        if txt.chars().count()>84 {s.push('…');}
        s
    }).unwrap_or_else(|| "Abra o guia e veja exemplos práticos.".into())
}
fn icone_topico(id: &str) -> &'static str {
    match id {
        "comecar" => "file-plus", "formulas" => "math-function",
        "preencher" => "edit", "abas" => "layout-sidebar",
        "formatar" => "bold", "tabelas" => "table",
        "graficos" => "chart-bar", "dados" => "filter",
        "macros" => "code", "arquivos" => "folder-open",
        "config" => "settings", "funcoes" => "math-function",
        "atalhos" => "keyboard", "slides"=>"layout-grid", "elementos"|"formas"=>"forms", "texto"=>"text-size", "apresentar"=>"player-play", "configuracoes"=>"settings", _ => "help",
    }
}
fn guia_do_topico(s: usize, i: usize, t: &Topico) -> Guia {
    Guia {
        titulo: t.titulo.clone(),
        descricao: resumo(t),
        icone: icone_topico(&t.id),
        destino: Evento::IrTopico(s,i),
    }
}
fn cartao(guia: &Guia) -> Element<'static, Evento> {
    consulta::cartao(guia.icone, guia.titulo.clone(), guia.descricao.clone(), Some(guia.destino.clone()))
}
fn grade_de_cards(guias: &[Guia], colunas: usize) -> Element<'static, Evento> {
    consulta::grade(guias.iter().map(cartao).collect(), colunas)
}
fn rotulo_nav(rotulo: &'static str) -> Element<'static,Evento> {
    consulta::rotulo_grupo(rotulo)
}
fn item_nav(rotulo: String, icone: &str, ativo: bool, destino: Evento) -> Element<'static,Evento> {
    consulta::item_nav(rotulo, icone, ativo, Some(destino))
}
fn navega(catalogo: &Catalogo, estado: &Estado) -> Element<'static,Evento> {
    let mut nav=column![].spacing(2);
    nav=nav.push(item_nav(crate::idioma::t("Início").into(),"file-plus",
        estado.secao==0 && estado.topico.is_none() && estado.busca.trim().is_empty(),
        Evento::IrSecao(0)));
    let Some(primeiro)=catalogo.secoes.first() else {return nav.into();};
    let indices=|ids: &[&str]| -> Vec<usize> {
        primeiro.topicos.iter().enumerate()
            .filter_map(|(i,t)|ids.contains(&t.id.as_str()).then_some(i)).collect()
    };
    let basico=indices(&["comecar","abas","preencher","formulas","formatar","tabelas","graficos","slides","elementos","texto","formas"]);
    let avancado=indices(&["dados","macros","apresentar"]);
    let outros=indices(&["arquivos","config","configuracoes"]);
    let outros_indices=(0..primeiro.topicos.len()).filter(|i|
        !basico.contains(i)&&!avancado.contains(i)&&!outros.contains(i)
    ).collect::<Vec<_>>();
    let group=|nav: pelicano::Column<'static,Evento>,
                   titulo: &'static str, itens: &[usize]| {
        if itens.is_empty(){return nav;}
        let mut nav=nav.push(rotulo_nav(titulo));
        for &i in itens {
            if let Some(t)=primeiro.topicos.get(i) {
                nav=nav.push(item_nav(t.titulo.clone(),icone_topico(&t.id),
                    estado.busca.trim().is_empty()&&estado.secao==0&&estado.topico==Some(i),
                    Evento::IrTopico(0,i)));
            }
        }
        nav
    };
    nav=group(nav,crate::idioma::t("BÁSICO"),&basico);
    nav=group(nav,crate::idioma::t("AVANÇADO"),&avancado);
    for (i,s) in catalogo.secoes.iter().enumerate().skip(1) {
        if s.id=="atalhos" {continue;}
        nav=nav.push(item_nav(s.titulo.clone(),icone_topico(&s.id),
            estado.busca.trim().is_empty() && estado.secao==i && estado.topico.is_none(),
            Evento::IrSecao(i)));
    }
    nav=group(nav,crate::idioma::t("OUTROS"),&outros);
    for i in outros_indices {
        if let Some(t)=primeiro.topicos.get(i) {
            nav=nav.push(item_nav(t.titulo.clone(),icone_topico(&t.id),
                estado.busca.trim().is_empty()&&estado.secao==0&&estado.topico==Some(i),
                Evento::IrTopico(0,i)));
        }
    }
    for (i,s) in catalogo.secoes.iter().enumerate().skip(1) {
        if s.id!="atalhos" {continue;}
        nav=nav.push(item_nav(s.titulo.clone(),"keyboard",
            estado.busca.trim().is_empty()&&estado.secao==i&&estado.topico.is_none(),
            Evento::IrSecao(i)));
    }
    nav.into()
}
fn home(catalogo: &Catalogo) -> Element<'static, Evento> {
    let Some(secao)=catalogo.secoes.first() else {return text(crate::idioma::t("Ajuda indisponível.")).into()};
    let intro=secao.introducao.clone();
    let hero=row![
        icone_hero(),
        column![
            texto_negrito(secao.titulo.clone(), 24.),
            text(intro).size(13).color(tema::COR_TEXTO_MUTED()).line_height(1.45),
        ].spacing(12).width(Length::FILL),
        if secao.topicos.iter().any(|t|t.id=="apresentar") {ilustra_apresentacao()}else{ilustra_planilha()},
    ].spacing(18).align(Align::Center);
    let ordem=["comecar","formatar","tabelas","graficos","dados","macros","funcoes","config","atalhos"];
    let mut itens=Vec::new();
    for id in ordem {
        if let Some((i,t))=secao.topicos.iter().enumerate().find(|(_,t)|t.id==id) {
            itens.push(guia_do_topico(0,i,t));
        } else if let Some((i,s))=catalogo.secoes.iter().enumerate().find(|(_,s)|s.id==id) {
            itens.push(Guia {
                titulo:s.titulo.clone(), descricao:s.introducao.chars().take(84).collect(),
                icone:icone_topico(id), destino:Evento::IrSecao(i)
            });
        }
    }
    // Inclui todos os assuntos dos outros editores, além da ordem histórica do Sheet.
    for (i,t) in secao.topicos.iter().enumerate().filter(|(_,t)|!ordem.contains(&t.id.as_str())) {
        itens.push(guia_do_topico(0,i,t));
    }
    // Guia rápido abre sua explicação correspondente; não cria arquivos sem confirmação.
    let apresentacao=secao.topicos.iter().any(|t|t.id=="apresentar");
    let atalhos=if apresentacao {vec![
        ("file-plus","Criar apresentação","comecar"),("folder-open","Abrir arquivo","arquivos"),
        ("device-floppy","Salvar","arquivos"),("forms","Inserir forma","elementos"),("player-play","Apresentar","apresentar"),
    ]} else {vec![
        ("file-plus","Criar planilha","comecar"),
        ("folder-open","Abrir arquivo","arquivos"),
        ("device-floppy","Salvar","arquivos"),
        ("printer","Imprimir","arquivos"),
        ("filter","Importar dados","dados"),
        ("math-function","Inserir função","formulas"),
        ("table","Criar tabela","tabelas"),
    ]};
    let mut quick=row![].spacing(7);
    for &(icone,titulo,topico) in &atalhos {
        if let Some((i,_))=secao.topicos.iter().enumerate().find(|(_,t)|t.id==topico) {
            quick=quick.push(button(row![
                icones::icone(icone,14.),
                text(titulo).size(11.5),
            ].spacing(6).align(Align::Center))
                .style(estilo_card).padding([8u16,9])
                .on_press(Evento::IrTopico(0,i)));
        }
    }
    column![
        hero,
        texto_negrito("Tópicos principais", 18.),
        grade_de_cards(&itens,3),
        consulta::divisor(),
        texto_negrito("Guias rápidos", 18.),
        quick,
    ].spacing(17).width(Length::FILL).into()
}
fn botao_copiar(texto: &str, copiado: Option<&str>) -> Element<'static, Evento> {
    let feito = copiado == Some(texto);
    crate::componentes::botao_icone(
        icones::icone(if feito { "check" } else { "copy" }, 16.),
        if feito { "Copiado" } else { "Copiar" },
        Some(Evento::Copiar(texto.to_owned())),
        feito,
    )
}
fn ver_topico(topico: &Topico, copiado: Option<&str>) -> Element<'static, Evento> {
    let mut body=column![texto_negrito(topico.titulo.clone(),23.)].spacing(13);
    for linha in &topico.linhas {
        let elemento:Element<'static,Evento>=match linha {
            Linha::Texto(s)=>text(s.clone()).size(14.).line_height(1.5).into(),
            Linha::Marcador(s)=>row![text("•"),text(s.clone()).size(14.).line_height(1.5)]
                .spacing(9).into(),
            Linha::Atalho{descricao,tecla}=>row![
                text(descricao.clone()).size(13.).width(Length::Fill(3)),
                text(tecla.clone()).size(13.).font(Font::MONO)
                    .width(Length::Fill(2)),
            ].spacing(9).into(),
            Linha::Exemplo(s)=>container(row![
                    text(s.clone()).size(13.).font(Font::MONO).width(Length::FILL),
                    botao_copiar(s, copiado),
                ].spacing(10).align(Align::Start))
                .padding(14.).width(Length::FILL)
                .style(|_|Style::default().background(tema::COR_PAINEL_FAIXA()).border(Border::rounded(8.0)))
                .into(),
        };
        body=body.push(elemento);
    }
    body.into()
}
fn conteudo(catalogo:&Catalogo,estado:&Estado)->Element<'static,Evento>{
    if !estado.busca.trim().is_empty() {
        let termo=normalizar(estado.busca.trim());
        let mut encontrados=Vec::new();
        for (s,secao) in catalogo.secoes.iter().enumerate() {
            for (i,topico) in secao.topicos.iter().enumerate() {
                if corresponde(&termo,topico) {
                    encontrados.push(guia_do_topico(s,i,topico));
                }
            }
        }
        if encontrados.is_empty() {
            return column![
                texto_negrito("Nenhum resultado encontrado", 21.),
                text(catalogo.nada_encontrado.clone()).size(13.)
                    .color(tema::COR_TEXTO_MUTED()),
            ].spacing(12).into();
        }
        return column![
            texto_negrito(format!("Resultados da busca ({})",encontrados.len()),22.),
            grade_de_cards(&encontrados,2),
        ].spacing(16).into();
    }
    let Some(secao)=catalogo.secoes.get(estado.secao) else {
        return text(crate::idioma::t("Ajuda indisponível.")).into();
    };
    if estado.secao==0&&estado.topico.is_none() {
        return home(catalogo);
    }
    if let Some(topico)=estado.topico.and_then(|t|secao.topicos.get(t)) {
        return column![
            text(secao.titulo.clone()).size(12.).color(tema::COR_TEXTO_MUTED()),
            ver_topico(topico, estado.copiado.as_deref()),
        ].spacing(13).into();
    }
    let guias=secao.topicos.iter().enumerate()
        .map(|(i,t)|guia_do_topico(estado.secao,i,t)).collect::<Vec<_>>();
    column![
        texto_negrito(secao.titulo.clone(),24.),
        text(secao.introducao.clone()).size(13.)
            .color(tema::COR_TEXTO_MUTED()).line_height(1.4),
        grade_de_cards(&guias,2),
    ].spacing(17).into()
}
fn paginacao(catalogo: &Catalogo,estado: &Estado)->Element<'static,Evento> {
    let previous=estado.secao>0||estado.topico.is_some();
    let is_last=catalogo.secoes.last()
        .is_some_and(|last|estado.secao+1>=catalogo.secoes.len()
            && estado.topico==last.topicos.len().checked_sub(1));
    row![
        button(text(format!("‹ {}",catalogo.anterior)).size(12.))
            .padding([8u16,13]).style(estilo_card)
            .on_press_maybe(previous.then_some(Evento::Anterior)),
        horizontal_space(),
        button(text(format!("{} ›",catalogo.proximo)).size(12.))
            .padding([8u16,13]).style(crate::tema::estilo_botao_primario)
            .on_press_maybe((!is_last).then_some(Evento::Proximo)),
    ].spacing(10).align(Align::Center).into()
}
impl Catalogo {
    pub fn view<'a>(&'a self,estado:&'a Estado)->Element<'a,Evento> {
        let sidebar=consulta::lateral(column![navega(self,estado)].spacing(4), None);
        let pesquisa=row![
            icones::icone("search",18.),
            text_input(self.busca_rotulo.as_str(),estado.busca.as_str())
                .id(ID_BUSCA)
                .on_input(Evento::Buscar)
                .padding(8.).size(14.)
                .style(|_,_| Style::default().background(tema::COR_PAINEL()).text_color(tema::COR_TEXTO()))
                .width(Length::FILL),
            container(text("Ctrl + F").size(10.5).color(tema::COR_TEXTO_MUTED()))
                .padding([5u16,8]).style(|_|Style::default()
                    .background(tema::COR_PAINEL_FAIXA())
                    .border(Border::rounded(6.0).color(tema::COR_BORDA(),1.))),
        ].spacing(10).align(Align::Center);
        let busca=container(pesquisa).padding([6u16,12]).width(Length::FILL).style(barra);
        let rolagem=scrollable(container(conteudo(self,estado))
            .padding([18u16, 8]).width(Length::FILL));
        let leitor=column![busca,rolagem].spacing(10)
            .padding([16u16,20]).width(Length::FILL).height(Length::FILL)
            .push_maybe(estado.busca.trim().is_empty().then(|| paginacao(self,estado)));
        row![
            sidebar,
            container(leitor).width(Length::FILL)
                .height(Length::FILL)
                .style(|_|Style::default().background(tema::COR_PAINEL())),
        ].height(Length::FILL).width(Length::FILL).into()
    }
}

fn ilustra_apresentacao()->Element<'static,Evento>{
    let svg=format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 170 115\"><rect x=\"4\" y=\"8\" width=\"162\" height=\"96\" rx=\"10\" fill=\"{}\" stroke=\"#dce4e9\"/><rect x=\"12\" y=\"18\" width=\"30\" height=\"22\" rx=\"3\" fill=\"#dce4e9\"/><rect x=\"12\" y=\"46\" width=\"30\" height=\"22\" rx=\"3\" fill=\"#dce4e9\"/><rect x=\"50\" y=\"18\" width=\"106\" height=\"76\" rx=\"5\" fill=\"#fff\"/><path d=\"M65 40h60M65 53h45M65 66h55\" stroke=\"{}\" stroke-width=\"5\" stroke-linecap=\"round\"/></svg>",if tema::escuro(){"#33363b"}else{"#f1f3f4"},tema::acento_hex());
    pelicano::svg(icones::carregar(svg)).width(170.).height(115.).into()
}

#[cfg(test)]
mod testes_visual {
    use super::*;

    #[test]
    fn icone_do_cabecalho_nao_expande_altura_no_scroll_vertical() {
        // center_y recebe a dimensão final, não apenas o alinhamento.
        // Fill neste elemento provocava overflow em cosmic-text ao pressionar F1.
        let cabecalho = icone_hero();
        let (largura, altura) = cabecalho.widget().size();
        assert_eq!(largura, Length::Fixed(96.));
        assert_eq!(altura, Length::Fixed(96.));
    }
    #[test]
    fn scroll_ajuda_aceita_home_topico_busca_sem_fill_vertical() {
        // Reproduz a construção da view que ocorre imediatamente após F1.
        let catalogo=Catalogo{
            titulo:"Ajuda".into(), busca_rotulo:"Buscar".into(),
            nada_encontrado:"Nada".into(), anterior:"Anterior".into(),
            proximo:"Próximo".into(), nesta_secao:"Tópicos".into(),
            secoes:vec![Secao{
                id:"uso".into(), titulo:"Como usar".into(),
                introducao:"Manual de uso".into(),
                topicos:vec![Topico{
                    id:"comecar".into(), titulo:"Primeiros passos".into(),
                    linhas:vec![Linha::Texto("Leia a documentação".into())],
                }],
            }],
        };
        let mut estado=Estado::default();
        let _: Element<'_, Evento> = catalogo.view(&estado);
        estado.topico=Some(0);
        let _: Element<'_, Evento> = catalogo.view(&estado);
        estado.busca="primeiros".into();
        let _: Element<'_, Evento> = catalogo.view(&estado);
        estado.busca.clear();
        estado.topico=None;
        // Desenha a página inicial de verdade: layout com rolagem não pode estourar.
        let png=pelicano::capture(&catalogo.view(&estado), pelicano::Size::new(900., 600.), 1.0,
            &tema::tema(), &mut pelicano::Renderer::new(), None);
        assert!(!png.is_empty());
    }

    #[test]
    fn tela_sem_titulo_ajuda_duplicado_e_navegacao_reutilizavel() {
        let c=Catalogo{
            titulo:"Ajuda do FeatherSheet".into(), busca_rotulo:"Buscar".into(),
            nada_encontrado:"Nada encontrado".into(), anterior:"Anterior".into(),
            proximo:"Próximo".into(), nesta_secao:"Nesta seção".into(),
            secoes:vec![Secao{
                id:"uso".into(), titulo:"Como usar o FeatherSheet".into(),
                introducao:"Guia rápido da planilha".into(),
                topicos:vec![Topico{
                    id:"comecar".into(),titulo:"Primeiros passos".into(),
                    linhas:vec![Linha::Texto("Abra uma planilha".into())],
                }],
            }],
        };
        let e=Estado::default();
        let _:Element<'_,Evento>=c.view(&e);
        assert_eq!(c.pagina("comecar"),Some((0,Some(0))));
        assert!(icones::existe("file-plus"));
        assert!(icones::existe("search"));
    }
}
