//! Monitor F12 da suíte: Memória, Console e Métricas.
//!
//! Tudo do monitor mora aqui — leitura de memória e CPU do processo, gráfico,
//! console com busca e filtro, abas e rodapé. O aplicativo só entrega o que é
//! dele: as mensagens do console ([`Monitor::registrar`]) e as métricas do
//! documento ([`Metricas`]). Nada é gravado em disco nem sai da máquina.
use crate::tema::*;
use pelicano::{button, column, container, program, row, scrollable, stack, svg, text, text_input, Align, Background,
    Border, Color, Element, Font, Length, Point, Program, Rect, Renderer, Space, Style, Svg, Task, Theme};
use std::{collections::VecDeque, time::Instant};

const MAX_REGISTROS: usize = 1_000;
const MAX_BYTES: usize = 512 * 1024;
const MAX_MENSAGEM: usize = 4_096;
const MAX_AMOSTRAS: usize = 80;
const ID_CONSOLE: &str = "feather-ui-monitor-console";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nivel {
    Info,
    Aviso,
    Erro,
}

impl Nivel {
    fn rotulo(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Aviso => "AVISO",
            Self::Erro => "ERRO",
        }
    }

    fn cor(self) -> Color {
        match self {
            Self::Info => COR_AZUL_ACENTO(),
            Self::Aviso => {
                if escuro() { Color::rgb8(245, 193, 97) } else { Color::rgb8(135, 85, 12) }
            }
            Self::Erro => {
                if escuro() { Color::rgb8(255, 145, 145) } else { Color::rgb8(184, 45, 55) }
            }
        }
    }

    /// O nível de uma mensagem de estado, pelas palavras dela.
    pub fn da_mensagem(msg: &str) -> Self {
        let msg = msg.to_lowercase();
        if msg.starts_with("erro")
            || msg.starts_with("error")
            || msg.starts_with("failed")
            || msg.starts_with("could not")
            || msg.contains("invalid")
            || msg.starts_with("falha")
            || msg.starts_with("não foi possível")
            || msg.contains("inválid")
            || msg.contains("interrompida")
        {
            Self::Erro
        } else if msg.contains("cancelad")
            || msg.contains("cancel")
            || msg.starts_with("select ")
            || msg.starts_with("open ")
            || msg.starts_with("enter ")
            || msg.starts_with("selecione")
            || msg.starts_with("abra ")
            || msg.starts_with("informe ")
        {
            Self::Aviso
        } else {
            Self::Info
        }
    }
}

/// O que acontece dentro da janela do monitor. O aplicativo embrulha numa
/// mensagem própria e devolve em [`Monitor::atualizar`].
#[derive(Debug, Clone)]
pub enum Evento {
    Aba(usize),
    Busca(String),
    Filtro(Option<Nivel>),
    Limpar,
    Copiar,
    Acompanhar,
    AlternarAmostras,
    Amostras(usize),
}

pub struct Registro {
    pub tempo: String,
    pub nivel: Nivel,
    pub origem: &'static str,
    pub mensagem: String,
}

/// O console local da sessão, com teto de memória.
pub struct Console {
    inicio: Instant,
    registros: VecDeque<Registro>,
    bytes: usize,
    descartados: usize,
    revisao: u64,
    busca: String,
    filtro: Option<Nivel>,
    acompanhar: bool,
}

impl Console {
    pub fn novo(aplicativo: &str) -> Self {
        let mut console = Self {
            inicio: Instant::now(),
            registros: VecDeque::new(),
            bytes: 0,
            descartados: 0,
            revisao: 0,
            busca: String::new(),
            filtro: None,
            acompanhar: true,
        };
        console.registrar(Nivel::Info, "Sistema",
            &crate::idioma::tf("{} iniciado. Console local da sessão pronto.", &[&aplicativo]));
        console
    }

    pub fn registrar(&mut self, nivel: Nivel, origem: &'static str, mensagem: &str) {
        if mensagem.is_empty() {
            return;
        }
        let mut fim = mensagem.len().min(MAX_MENSAGEM);
        while !mensagem.is_char_boundary(fim) {
            fim -= 1;
        }
        let truncada = fim < mensagem.len();
        let mut mensagem = mensagem[..fim].to_owned();
        // O truncamento usa a fronteira UTF-8 para preservar acentos.
        if truncada {
            mensagem.push_str(crate::idioma::t(" … [registro limitado]"));
        }
        let segundos = self.inicio.elapsed().as_secs();
        self.revisao = self.revisao.wrapping_add(1);
        self.bytes += mensagem.len();
        self.registros.push_back(Registro {
            tempo: format!("{:02}:{:02}:{:02}", segundos / 3600, segundos / 60 % 60, segundos % 60),
            nivel,
            origem,
            mensagem,
        });
        while self.registros.len() > MAX_REGISTROS || self.bytes > MAX_BYTES {
            if let Some(antigo) = self.registros.pop_front() {
                self.bytes -= antigo.mensagem.len();
                self.descartados += 1;
            }
        }
    }

    /// Uma saída de várias linhas (a de uma macro, por exemplo), cada linha
    /// com o nível que as palavras dela indicam.
    pub fn registrar_saida(&mut self, origem: &'static str, saida: &str) {
        // Mantém no máximo o trecho final de uma saída extensa, sem duplicá-la inteira.
        let mut inicio = saida.len().saturating_sub(MAX_BYTES / 2);
        while !saida.is_char_boundary(inicio) {
            inicio += 1;
        }
        if inicio > 0 {
            self.registrar(Nivel::Aviso, origem, crate::idioma::t("Saída extensa: exibindo o trecho final."));
        }
        for linha in saida[inicio..].lines().filter(|s| !s.is_empty()) {
            self.registrar(Nivel::da_mensagem(linha), origem, linha);
        }
    }

    /// Sobe a cada registro: quem chama compara para saber se algo entrou.
    pub fn revisao(&self) -> u64 {
        self.revisao
    }

    pub fn registros(&self) -> impl Iterator<Item = &Registro> {
        self.registros.iter()
    }

    pub fn erros(&self) -> usize {
        self.registros.iter().filter(|r| r.nivel == Nivel::Erro).count()
    }

    fn visiveis(&self) -> impl Iterator<Item = &Registro> {
        let busca = self.busca.trim().to_lowercase();
        self.registros.iter().filter(move |r| {
            self.filtro.map_or(true, |n| n == r.nivel)
                && (busca.is_empty()
                    || r.mensagem.to_lowercase().contains(&busca)
                    || r.origem.to_lowercase().contains(&busca))
        })
    }

    fn texto_visivel(&self) -> String {
        self.visiveis()
            .map(|r| format!("[{}] [{}] [{}] {}", r.tempo, r.nivel.rotulo(), r.origem, r.mensagem))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn rolar_ao_fim<M: Send + 'static>(&self) -> Task<M> {
        if self.acompanhar {
            Task::snap_to(ID_CONSOLE, Point::new(0.0, 1.0))
        } else {
            Task::none()
        }
    }
}

/// O que o aplicativo mostra na aba Métricas. "Erros no console" e "Tempo
/// de sessão" o monitor acrescenta sozinho.
#[derive(Default)]
pub struct Metricas {
    /// Cartões do alto: (rótulo, valor, detalhe).
    pub cartoes: Vec<(String, String, String)>,
    /// Linhas de detalhe: (rótulo, valor).
    pub linhas: Vec<(String, String)>,
    /// Texto do rodapé quando há trabalho pesado em curso ("Importando…").
    pub ocupado: Option<String>,
}

/// O monitor inteiro: amostras do processo, console e estado da janela.
pub struct Monitor {
    aba: usize,
    rss_kb: u64,
    pico_kb: u64,
    cpu: f32,
    cpu_anterior: Option<(u64, u64)>,
    historico: Vec<u64>,
    /// CPU (%) de cada amostra, junto com `historico`.
    historico_cpu: Vec<f32>,
    amostras: usize,
    menu_amostras: bool,
    pub console: Console,
}

impl Monitor {
    pub fn novo(aplicativo: &str) -> Self {
        Self {
            aba: 0,
            rss_kb: 0,
            pico_kb: 0,
            cpu: 0.0,
            cpu_anterior: None,
            historico: Vec::new(),
            historico_cpu: Vec::new(),
            amostras: MAX_AMOSTRAS,
            menu_amostras: false,
            console: Console::novo(aplicativo),
        }
    }

    pub fn registrar(&mut self, nivel: Nivel, origem: &'static str, mensagem: &str) {
        self.console.registrar(nivel, origem, mensagem);
    }

    /// O console da sessão (para conferir o que foi registrado).
    pub fn console(&self) -> &Console {
        &self.console
    }

    /// A aba aberta: 0 Memória, 1 Console, 2 Métricas.
    pub fn aba(&self) -> usize {
        self.aba
    }

    /// RSS da última amostra, em KiB (0 se o sistema não informa).
    pub fn rss_kb(&self) -> u64 {
        self.rss_kb
    }

    /// CPU da última amostra (% da capacidade total); `None` antes da segunda amostra.
    pub fn cpu(&self) -> Option<f32> {
        self.cpu_anterior.map(|_| self.cpu)
    }

    pub fn amostras(&self) -> usize {
        self.amostras
    }

    pub fn menu_amostras_aberto(&self) -> bool {
        self.menu_amostras
    }

    /// Lê memória e CPU do processo. Chamar uma vez por segundo.
    pub fn amostrar(&mut self) {
        if let Some(kb) = rss_atual_kb() {
            self.rss_kb = kb;
            self.pico_kb = self.pico_kb.max(kb);
            self.historico.push(kb);
            if self.historico.len() > MAX_AMOSTRAS {
                self.historico.remove(0);
            }
        }
        if let Some((proc, total)) = cpu_atual() {
            if let Some((proc_antes, total_antes)) = self.cpu_anterior {
                self.cpu = (proc.saturating_sub(proc_antes) as f32
                    / total.saturating_sub(total_antes).max(1) as f32 * 100.0).clamp(0.0, 100.0);
                self.historico_cpu.push(self.cpu);
                if self.historico_cpu.len() > MAX_AMOSTRAS {
                    self.historico_cpu.remove(0);
                }
            }
            self.cpu_anterior = Some((proc, total));
        }
    }

    /// Rola o console até o fim, se ele estiver aberto e acompanhando.
    pub fn acompanhar<M: Send + 'static>(&self) -> Task<M> {
        if self.aba == 1 { self.console.rolar_ao_fim() } else { Task::none() }
    }

    pub fn atualizar<M: Send + 'static>(&mut self, evento: Evento) -> Task<M> {
        match evento {
            Evento::Aba(aba) => {
                self.aba = aba.min(2);
                self.menu_amostras = false;
                return self.acompanhar();
            }
            Evento::Busca(v) => self.console.busca = v,
            Evento::Filtro(v) => self.console.filtro = v,
            Evento::Limpar => {
                self.console.registros.clear();
                self.console.bytes = 0;
                self.console.descartados = 0;
            }
            Evento::Copiar => return Task::write_clipboard(self.console.texto_visivel()),
            Evento::Acompanhar => {
                self.console.acompanhar = !self.console.acompanhar;
                return self.console.rolar_ao_fim();
            }
            Evento::AlternarAmostras => self.menu_amostras = !self.menu_amostras,
            Evento::Amostras(n) => {
                if [20, 40, 80].contains(&n) {
                    self.amostras = n;
                }
                self.menu_amostras = false;
            }
        }
        Task::none()
    }

    /// A janela inteira. `msg` embrulha os eventos na mensagem do aplicativo.
    pub fn view<'a, M: Clone + 'static>(
        &'a self,
        metricas: Metricas,
        msg: impl Fn(Evento) -> M + Clone + 'a,
    ) -> Element<'a, M> {
        let mut abas = row![].spacing(14).align(Align::Center);
        for (i, (titulo, icone)) in [
            (crate::idioma::t("Memória"), "device-desktop"),
            (crate::idioma::t("Console"), "terminal"),
            (crate::idioma::t("Métricas"), "chart-bar"),
        ].into_iter().enumerate() {
            let selecionado = self.aba == i;
            let cor = if selecionado { verde() } else { COR_TEXTO_MUTED() };
            let conteudo = row![
                crate::icones::icone(icone, 21.0).color(cor),
                text(titulo).size(14),
            ].spacing(10).align(Align::Center);
            let aba = button(conteudo)
                .padding([11, 18])
                .style(move |tema, estado| {
                    let mut estilo = estilo_botao_secundario(tema, estado);
                    estilo.border = Border::rounded(9.0);
                    estilo.background = selecionado.then(|| Background::Color(COR_AZUL_FUNDO()));
                    estilo.text_color(if selecionado { verde() } else { COR_TEXTO_MUTED() })
                })
                .on_press(msg(Evento::Aba(i)));
            // Largura fixa: um traço FILL fazia a primeira aba tomar a linha toda e espremer as outras.
            let detalhe = container(Space::new(48.0, 3.0))
                .style(move |_| Style { background: selecionado.then(|| Background::Color(verde())), ..Style::default().border(Border::rounded(3.0)) });
            abas = abas.push(column![aba, detalhe].spacing(0).align(Align::Center));
        }
        let ocupado = metricas.ocupado.clone();
        let corpo = match self.aba {
            1 => self.view_console(msg.clone()),
            2 => self.view_metricas(metricas),
            _ => self.view_memoria(msg.clone()),
        };
        let rodape = row![
            text(ocupado.unwrap_or_else(|| crate::idioma::t("Sessão ativa").into())).size(11).color(COR_TEXTO_MUTED()),
            Space::new(Length::FILL, 0.0),
            container(Space::new(9.0, 9.0))
                .style(|_| Style::default().background(verde()).border(Border::rounded(5.0))),
            text(crate::idioma::t("Registros locais · atualização a cada segundo")).size(11).color(COR_TEXTO_MUTED()),
        ].spacing(9).align(Align::Center);
        container(column![abas, divisor(), corpo, divisor(), rodape].spacing(13))
            .padding([18, 22])
            .width(Length::FILL)
            .height(Length::FILL)
            .into()
    }

    pub fn view_console<'a, M: Clone + 'static>(&'a self, msg: impl Fn(Evento) -> M + Clone + 'a) -> Element<'a, M> {
        let console = &self.console;
        let mut filtros = row![].spacing(4);
        for (nome, nivel) in [
            (crate::idioma::t("Todos"), None),
            (crate::idioma::t("Informações"), Some(Nivel::Info)),
            (crate::idioma::t("Avisos"), Some(Nivel::Aviso)),
            (crate::idioma::t("Erros"), Some(Nivel::Erro)),
        ] {
            filtros = filtros.push(
                button(text(nome).size(12))
                    .padding([6, 10])
                    .style(estilo_item_menu(console.filtro == nivel, true))
                    .on_press(msg(Evento::Filtro(nivel))),
            );
        }
        let ao_buscar = msg.clone();
        let busca = text_input(crate::idioma::t("Pesquisar mensagem ou origem…"), &console.busca)
            .size(13)
            .padding([9, 12])
            .style(estilo_entrada_suspensa)
            .on_input(move |v| ao_buscar(Evento::Busca(v)));
        let visiveis: Vec<_> = console.visiveis().collect();
        let total_visivel = visiveis.len();
        let mut linhas = column![].spacing(0);
        for r in visiveis {
            linhas = linhas
                .push(
                    container(
                        row![
                            text(&r.tempo).font(Font::MONO).size(11).color(COR_TEXTO_MUTED()).width(76.0),
                            text(r.nivel.rotulo()).size(10).color(r.nivel.cor()).width(46.0),
                            text(r.origem).size(11).color(COR_TEXTO_MUTED()).width(72.0),
                            text(&r.mensagem).font(Font::MONO).size(12).width(Length::FILL),
                        ].spacing(10),
                    )
                    .padding([9, 12])
                    .width(Length::FILL),
                )
                .push(divisor());
        }
        if total_visivel == 0 {
            linhas = linhas.push(
                container(
                    column![
                        text(crate::idioma::t("Nenhum registro para exibir")).size(17),
                        text(if console.registros.is_empty() {
                            crate::idioma::t("Novas operações, erros e saídas de macros aparecerão aqui.")
                        } else {
                            crate::idioma::t("Altere a busca ou o filtro para ver outros registros.")
                        })
                        .size(13)
                        .color(COR_TEXTO_MUTED()),
                    ].spacing(8),
                )
                .padding(28),
            );
        }
        let lista = container(scrollable(linhas).id(ID_CONSOLE).height(Length::FILL))
            .width(Length::FILL)
            .height(Length::FILL)
            .style(painel);
        let resumo = format!(
            "{}{}",
            crate::idioma::tf("{} de {} registros", &[&total_visivel, &console.registros.len()]),
            if console.descartados > 0 {
                crate::idioma::tf(" · {} antigos descartados", &[&console.descartados])
            } else {
                String::new()
            }
        );
        column![
            busca,
            row![
                filtros,
                Space::new(Length::FILL, 0.0),
                acao(crate::idioma::t("Copiar"), msg(Evento::Copiar)),
                acao(crate::idioma::t("Limpar"), msg(Evento::Limpar)),
            ].spacing(6).align(Align::Center),
            lista,
            row![
                text(resumo).size(11).color(COR_TEXTO_MUTED()),
                Space::new(Length::FILL, 0.0),
                acao(
                    if console.acompanhar { crate::idioma::t("Acompanhar: ligado") } else { crate::idioma::t("Acompanhar: desligado") },
                    msg(Evento::Acompanhar),
                ),
            ].spacing(8).align(Align::Center),
        ]
        .spacing(10)
        .height(Length::FILL)
        .into()
    }

    pub fn view_memoria<'a, M: Clone + 'static>(&'a self, msg: impl Fn(Evento) -> M + Clone + 'a) -> Element<'a, M> {
        self.view_memoria_com(msg, Vec::new(), None)
    }

    /// Como `view_memoria`, com linhas do aplicativo embaixo dos gráficos ("Páginas desenhadas", "3 · 16 MB")
    /// e um botão de ação opcional ("Liberar cache de páginas").
    pub fn view_memoria_com<'a, M: Clone + 'static>(
        &'a self,
        msg: impl Fn(Evento) -> M + Clone + 'a,
        extras: Vec<(String, String)>,
        acao: Option<(String, M)>,
    ) -> Element<'a, M> {
        let mib = |kb: u64| match kb {
            0 => "—".into(),
            // "1226.0 MiB" não cabe no cartão; acima de 1 GiB mostra "1.2 GiB".
            k if k >= 1024 * 1024 => format!("{:.1} GiB", k as f64 / (1024. * 1024.)),
            k => format!("{:.0} MiB", k as f64 / 1024.),
        };
        let porcentagem_rss = percentual_rss(self.rss_kb, ram_total_kb());
        let porcentagem_cpu = self.cpu_anterior.map(|_| self.cpu);
        let cartoes = row![
            cartao_destaque(crate::idioma::t("Memória em uso"), mib(self.rss_kb), crate::idioma::t("RAM residente do processo"),
                "device-desktop", verde(), Some((porcentagem_rss, "#2ab66f"))),
            cartao_destaque(crate::idioma::t("Pico observado"), mib(self.pico_kb), crate::idioma::t("Maior amostra desta sessão"),
                "chart-line", azul(), None),
            cartao_destaque("CPU",
                if self.cpu_anterior.is_some() { format!("{:.1}%", self.cpu) } else { "—".into() },
                crate::idioma::t("Fração da capacidade total"), "settings", roxo(), Some((porcentagem_cpu, "#8051d9"))),
        ].spacing(14);

        let titulo = row![
            crate::icones::icone("chart-line", 20.0).color(verde()),
            text(crate::idioma::t("Recursos")).size(16),
            Space::new(Length::FILL, 0.0),
            button(row![
                text(crate::idioma::tf("Últimas {} amostras", &[&self.amostras])).size(11),
                crate::icones::icone("chevron-down", 14.0)
                    .color(COR_TEXTO_MUTED()),
            ].spacing(10).align(Align::Center))
                .padding([8, 11])
                .style(estilo_botao_secundario)
                .on_press(msg(Evento::AlternarAmostras)),
        ].spacing(11).align(Align::Center);
        let item = |rotulo: String, valor: String| -> Element<'a, M> {
            row![text(rotulo).size(12).color(COR_TEXTO_MUTED()), Space::new(Length::FILL, 0.0), text(valor).size(12)]
                .spacing(8).into()
        };
        let ultimas = |v: &[f32]| v[v.len().saturating_sub(self.amostras)..].to_vec();
        let ram: Vec<f32> = self.historico.iter().map(|&kb| kb as f32 / 1024.0).collect();
        let maior_ram = ram.iter().copied().fold(1.0, f32::max) * 1.1;
        let logicos = std::thread::available_parallelism().map_or(0, |n| n.get());
        let fisicos = nucleos_fisicos().unwrap_or(logicos);
        let cpu = if self.cpu_anterior.is_some() { format!("{:.0}%", self.cpu) } else { "—".into() };
        let mut corpo = column![
            titulo,
            item(crate::idioma::t("Memória RAM").into(), format!("{} · pico {}", mib(self.rss_kb), mib(self.pico_kb))),
            linha(ultimas(&ram), maior_ram, verde()),
            item("CPU".into(), format!("{cpu} · {fisicos} núcleos · {logicos} threads")),
            linha(ultimas(&self.historico_cpu), 100.0, roxo()),
        ].spacing(10);
        if let Some(t) = threads_atuais() {
            corpo = corpo.push(item(crate::idioma::t("Threads do app").into(), t.to_string()));
        }
        for (rotulo, valor) in extras {
            corpo = corpo.push(item(rotulo, valor));
        }
        if let Some((rotulo, m)) = acao {
            corpo = corpo.push(button(text(rotulo).size(12).color(COR_AZUL_ACENTO())).padding(0)
                .style(|_, _| Style::default()).on_press(m));
        }
        let grafico_base = container(corpo)
            .padding(16)
            .width(Length::FILL)
            .height(Length::FILL)
            .style(painel)
            // Janela baixa demais para o gráfico: corta em vez de vazar sobre o banner.
            .clip(true);
        let grafico: Element<'a, M> = if self.menu_amostras {
            let mut opcoes = column![].spacing(2).padding(5);
            for n in [20, 40, 80] {
                opcoes = opcoes.push(
                    button(row![
                        text(crate::idioma::tf("Últimas {} amostras", &[&n])).size(12),
                        Space::new(Length::FILL, 0.0),
                        if self.amostras == n {
                            crate::icones::icone("circle-check", 15.0)
                                .color(verde())
                        } else {
                            crate::icones::icone("chevron-down", 15.0)
                                .color(COR_TEXTO_MUTED())
                        },
                    ].spacing(8).align(Align::Center))
                        .width(Length::FILL)
                        .padding([8, 10])
                        .style(estilo_botao_secundario)
                        .on_press(msg(Evento::Amostras(n))),
                );
            }
            let menu = container(opcoes).width(211.0).style(painel);
            stack![
                grafico_base,
                column![
                    Space::new(0.0, 57.0),
                    row![Space::new(Length::FILL, 0.0), menu],
                ]
            ].width(Length::FILL).height(Length::FILL).into()
        } else {
            grafico_base.into()
        };
        // Sem rolagem: o histórico ocupa o que sobra da janela abaixo dos
        // cartões, e encolhe com ela.
        column![cartoes, grafico, banner_informativo()].spacing(12)
            .height(Length::FILL)
            .into()
    }

    pub fn view_metricas<'a, M: 'static>(&'a self, metricas: Metricas) -> Element<'a, M> {
        let mut cartoes = row![].spacing(12);
        for (rotulo, valor, detalhe) in metricas.cartoes {
            cartoes = cartoes.push(cartao(rotulo, valor, detalhe));
        }
        cartoes = cartoes.push(cartao(crate::idioma::t("Erros no console").into(), self.console.erros().to_string(),
            crate::idioma::t("No histórico retido").into()));
        let mut corpo = column![cartoes].spacing(12);
        let sessao = self.console.inicio.elapsed().as_secs();
        let linhas = metricas.linhas.into_iter()
            .chain([(crate::idioma::t("Tempo de sessão").into(), crate::idioma::tf("{} min {} s", &[&(sessao / 60), &(sessao % 60)]))]);
        for (rotulo, valor) in linhas {
            corpo = corpo.push(
                container(
                    row![
                        text(rotulo).size(13).color(COR_TEXTO_MUTED()).width(160.0),
                        text(valor).size(13).width(Length::FILL),
                    ].spacing(12),
                )
                .padding(16)
                .width(Length::FILL)
                .style(painel),
            );
        }
        scrollable(corpo).height(Length::FILL).into()
    }
}

fn painel(_: &Theme) -> Style {
    Style::default().background(COR_PAINEL()).border(Border::rounded(14.0).color(COR_BORDA_SUAVE(), 1.0)).text_color(COR_TEXTO())
}

fn divisor<'a, M: 'static>() -> Element<'a, M> {
    container(Space::new(Length::FILL, 1.0)).width(Length::FILL).style(estilo_linha).into()
}

fn cartao<'a, M: 'static>(rotulo: String, valor: String, detalhe: String) -> Element<'a, M> {
    container(
        column![
            text(rotulo).size(12).color(COR_TEXTO_MUTED()),
            text(valor).size(25),
            text(detalhe).size(11).color(COR_TEXTO_MUTED()),
        ].spacing(7),
    )
    .padding(18)
    .width(Length::FILL)
    .style(painel)
    .into()
}

fn verde() -> Color {
    if escuro() { Color::rgb8(92, 214, 145) } else { Color::rgb8(21, 137, 67) }
}

fn azul() -> Color {
    if escuro() { Color::rgb8(123, 194, 246) } else { Color::rgb8(19, 135, 124) }
}

fn roxo() -> Color {
    if escuro() { Color::rgb8(189, 155, 255) } else { Color::rgb8(127, 66, 210) }
}

/// RSS do processo em KiB (Linux: `/proc/self/status`).
#[cfg(not(windows))]
fn rss_atual_kb() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status").ok()?
        .lines().find(|l| l.starts_with("VmRSS:"))?
        .split_whitespace().nth(1)?.parse().ok()
}

/// Threads do processo (Linux: `/proc/self/status`).
#[cfg(not(windows))]
fn threads_atuais() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status").ok()?
        .lines().find(|l| l.starts_with("Threads:"))?
        .split_whitespace().nth(1)?.parse().ok()
}

#[cfg(windows)]
fn threads_atuais() -> Option<u64> {
    None
}

/// Núcleos físicos (Linux: pares únicos `physical id` + `core id` do `/proc/cpuinfo`); `None` sem a informação.
#[cfg(not(windows))]
fn nucleos_fisicos() -> Option<usize> {
    contar_nucleos(&std::fs::read_to_string("/proc/cpuinfo").ok()?)
}

#[cfg(windows)]
fn nucleos_fisicos() -> Option<usize> {
    None
}

#[cfg_attr(windows, allow(dead_code))]
fn contar_nucleos(cpuinfo: &str) -> Option<usize> {
    let mut pares = std::collections::HashSet::new();
    for bloco in cpuinfo.split("\n\n") {
        let campo = |nome: &str| bloco.lines().find(|l| l.starts_with(nome))?.split(':').nth(1).map(|v| v.trim().to_owned());
        if let (Some(f), Some(c)) = (campo("physical id"), campo("core id")) { pares.insert((f, c)); }
    }
    (!pares.is_empty()).then_some(pares.len())
}

/// (tempo de CPU do processo, tempo total do sistema), em ticks.
#[cfg(not(windows))]
fn cpu_atual() -> Option<(u64, u64)> {
    let proc = std::fs::read_to_string("/proc/self/stat").ok()?;
    let campos: Vec<&str> = proc.rsplit_once(')')?.1.split_whitespace().collect();
    let processo = campos.get(11)?.parse::<u64>().ok()? + campos.get(12)?.parse::<u64>().ok()?;
    let total = std::fs::read_to_string("/proc/stat").ok()?.lines().next()?.split_whitespace()
        .skip(1).filter_map(|v| v.parse::<u64>().ok()).sum();
    Some((processo, total))
}

/// Total físico Linux lido uma vez, para NÃO inventar percentual RSS.
/// No Windows vem do `GlobalMemoryStatusEx`; sem fonte nativa (macOS), o medidor exibe '—'.
pub fn ram_total_kb() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        static TOTAL: std::sync::OnceLock<Option<u64>> = std::sync::OnceLock::new();
        *TOTAL.get_or_init(|| {
            std::fs::read_to_string("/proc/meminfo").ok()?
                .lines().find_map(|line| {
                    line.strip_prefix("MemTotal:")?.split_whitespace().next()?.parse::<u64>().ok()
                }).filter(|&n| n > 0)
        })
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        // SAFETY: estrutura zerada com o tamanho preenchido, como a API pede.
        let mut m: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
        m.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        (unsafe { GlobalMemoryStatusEx(&mut m) } != 0 && m.ullTotalPhys > 0).then(|| m.ullTotalPhys / 1024)
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    { None }
}

/// Windows (16/10/2026): sem `/proc`, o medidor ficava em "—". A memória é
/// o conjunto de trabalho do processo, como o Gerenciador de Tarefas mostra.
#[cfg(windows)]
fn rss_atual_kb() -> Option<u64> {
    use windows_sys::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    // SAFETY: estrutura zerada, tamanho correto e o pseudo-handle do próprio processo.
    let mut c: PROCESS_MEMORY_COUNTERS = unsafe { std::mem::zeroed() };
    let tamanho = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    c.cb = tamanho;
    (unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, tamanho) } != 0)
        .then(|| c.WorkingSetSize as u64 / 1024)
}

/// Windows: (núcleo + usuário do processo, núcleo + usuário do sistema), em
/// unidades de 100 ns. O tempo de núcleo do sistema já inclui o ocioso, e as
/// duas somas valem para todos os processadores, como no `/proc/stat`.
#[cfg(windows)]
fn cpu_atual() -> Option<(u64, u64)> {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes, GetSystemTimes};
    let valor = |f: &FILETIME| (u64::from(f.dwHighDateTime) << 32) | u64::from(f.dwLowDateTime);
    let zero = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let (mut criacao, mut saida, mut nucleo, mut usuario) = (zero, zero, zero, zero);
    // SAFETY: ponteiros para estruturas locais válidas.
    if unsafe { GetProcessTimes(GetCurrentProcess(), &mut criacao, &mut saida, &mut nucleo, &mut usuario) } == 0 {
        return None;
    }
    let (mut ocioso, mut nucleo_s, mut usuario_s) = (zero, zero, zero);
    // SAFETY: idem.
    if unsafe { GetSystemTimes(&mut ocioso, &mut nucleo_s, &mut usuario_s) } == 0 {
        return None;
    }
    Some((valor(&nucleo) + valor(&usuario), valor(&nucleo_s) + valor(&usuario_s)))
}

fn percentual_rss(rss_kb: u64, total_kb: Option<u64>) -> Option<f32> {
    let total = total_kb.filter(|&n| n > 0)?;
    (rss_kb > 0).then_some((rss_kb as f64 / total as f64 * 100.0).clamp(0.0, 100.0) as f32)
}

/// SVG circular com um único arco contínuo e círculo de fundo.
/// O atributo pathLength não é aplicado de modo uniforme a dasharrays em
/// círculo pelo renderizador Iced/resvg, criando pontos verdes/roxos soltos.
/// Evitamos tracejados e calculamos um arco geométrico com o ângulo real.
fn rosca_svg(valor: Option<f32>, hex: &str) -> String {
    const CX: f32 = 50.0;
    const CY: f32 = 50.0;
    const RAIO: f32 = 38.0;
    let fundo = if escuro() { "#454a52" } else { "#e6ebef" };
    let base = format!(
        r##"<circle cx="{CX}" cy="{CY}" r="{RAIO}" fill="none" stroke="{fundo}" stroke-width="9"/>"##
    );
    let progresso = match valor.filter(|v| v.is_finite()).map(|v| v.clamp(0.0, 100.0)) {
        None => String::new(),
        Some(v) if v <= 0.0 => String::new(),
        Some(v) if v >= 100.0 => format!(
            r##"<circle cx="{CX}" cy="{CY}" r="{RAIO}" fill="none" stroke="{hex}" stroke-width="9"/>"##
        ),
        Some(v) => {
            // Inicia em 12 horas e cresce no sentido horário.
            let angulo = (-90.0 + 360.0 * v / 100.0).to_radians();
            let fim_x = CX + RAIO * angulo.cos();
            let fim_y = CY + RAIO * angulo.sin();
            let arco_grande = if v > 50.0 { 1 } else { 0 };
            // Em percentuais pequenos, tampas arredondadas ampliam a fatia.
            let ponta = if v < 4.0 { "butt" } else { "round" };
            format!(
                r##"<path d="M 50 12 A 38 38 0 {arco_grande} 1 {fim_x:.3} {fim_y:.3}" fill="none" stroke="{hex}" stroke-width="9" stroke-linecap="{ponta}"/>"##
            )
        }
    };
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="100" height="100">{base}{progresso}</svg>"##
    )
}

fn rosca<'a, M: 'static>(valor: Option<f32>, hex: &'static str) -> Element<'a, M> {
    const LADO: f32 = 72.0;
    let rotulo = valor.map(|v| format!("{v:.0}%")).unwrap_or_else(|| "—".into());
    let desenho = Svg::from_bytes(rosca_svg(valor, hex).as_bytes()).map(|s| svg(s).width(LADO).height(LADO));
    stack(desenho.map(Element::from))
        .push(container(text(rotulo).size(14).color(COR_TEXTO())).width(LADO).height(LADO).center())
        .width(LADO)
        .height(LADO)
        .into()
}

fn cartao_destaque<'a, M: 'static>(
    titulo: &'static str,
    valor: String,
    detalhe: &'static str,
    icone: &'static str,
    cor: Color,
    medidor: Option<(Option<f32>, &'static str)>,
) -> Element<'a, M> {
    // Ícone ao lado do título (não num selo à esquerda): num cartão de ~270 px
    // o selo e a rosca deixavam ~60 px ao valor, que quebrava sobre o detalhe.
    let conteudo = column![
        row![crate::icones::icone(icone, 16.0).color(cor), text(titulo).size(13).color(COR_TEXTO_MUTED()).no_wrap().ellipsis()]
            .spacing(7).align(Align::Center),
        text(valor).size(22).color(COR_TEXTO()).no_wrap(),
        text(detalhe).size(11).color(COR_TEXTO_MUTED()).no_wrap().ellipsis(),
    ].spacing(6).width(Length::FILL);
    let mut linha = row![conteudo].spacing(12).align(Align::Center);
    if let Some((pct, cor)) = medidor {
        linha = linha.push(rosca(pct, cor));
    }
    container(linha)
        .padding([14, 16])
        .width(Length::FILL)
        .height(118.0)
        .style(painel)
        .into()
}

fn banner_informativo<'a, M: 'static>() -> Element<'a, M> {
    let icone = crate::icones::icone("info-circle", 21.0).color(verde());
    container(row![
        icone,
        text(crate::idioma::t("O monitor acompanha a memória do aplicativo inteiro, incluindo a interface e os documentos abertos."))
            .size(12)
            .color(COR_TEXTO_MUTED()),
    ].spacing(13).align(Align::Center))
        .padding(16)
        .width(Length::FILL)
        .style(|_| Style::default().background(COR_PAINEL_FAIXA()).border(Border::rounded(12.0).color(COR_BORDA_SUAVE(), 1.0)))
        .into()
}

fn acao<'a, M: Clone + 'static>(rotulo: &'static str, mensagem: M) -> Element<'a, M> {
    button(text(rotulo).size(12))
        .padding([7, 12])
        .style(estilo_botao_secundario)
        .on_press(mensagem)
        .into()
}

/// Gráfico de linha pequeno do quadro Recursos: `valores` de 0 a `maximo`, o último à direita.
struct GraficoLinha {
    valores: Vec<f32>,
    maximo: f32,
    cor: Color,
}

fn linha<'a, M: 'static>(valores: Vec<f32>, maximo: f32, cor: Color) -> Element<'a, M> {
    container(program(GraficoLinha { valores, maximo, cor }).width(Length::FILL).height(Length::FILL))
        .padding([4, 6]).height(Length::FILL).style(painel).into()
}

impl<M> Program<M> for GraficoLinha {
    type State = ();

    fn draw(&self, _: &(), r: &mut Renderer, b: Rect, _: &Theme, _: Option<Point>) {
        if self.valores.len() < 2 || b.height < 4.0 {
            return;
        }
        let passo = b.width / (self.valores.len() - 1) as f32;
        let y = |v: f32| b.y + b.height - 2.0 - (v / self.maximo.max(f32::EPSILON)).clamp(0.0, 1.0) * (b.height - 4.0);
        let pontos: Vec<Point> = self.valores.iter().enumerate().map(|(i, &v)| Point::new(b.x + i as f32 * passo, y(v))).collect();
        r.stroke(&pontos, 1.5, self.cor);
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use pelicano::Size;

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    enum Teste {
        Monitor(Evento),
    }

    #[test]
    fn as_tres_abas_desenham() {
        let mut m = Monitor::novo("Teste");
        m.amostrar();
        for aba in 0..3 {
            let _: Task<Teste> = m.atualizar(Evento::Aba(aba));
            let e = m.view(Metricas::default(), Teste::Monitor);
            let png = pelicano::capture(&e, Size::new(900.0, 640.0), 1.0, &Theme::light(), &mut Renderer::new(), None);
            assert!(!png.is_empty());
        }
    }

    #[test]
    fn nucleos_fisicos_contam_pares_unicos() {
        let info = "processor\t: 0\nphysical id\t: 0\ncore id\t: 0\n\nprocessor\t: 1\nphysical id\t: 0\ncore id\t: 0\n\n\
            processor\t: 2\nphysical id\t: 0\ncore id\t: 1\n";
        assert_eq!(contar_nucleos(info), Some(2));
        assert_eq!(contar_nucleos("processor\t: 0\n"), None);
    }

    #[test]
    fn grafico_desenha_linha_sem_publicar() {
        let g = GraficoLinha { valores: vec![10.0, 20.0, 40.0], maximo: 44.0, cor: Color::BLACK };
        let mut fora: Vec<Teste> = Vec::new();
        let b = Rect::new(Point::ORIGIN, Size::new(400.0, 200.0));
        let c = Program::<Teste>::update(&g, &mut (), &pelicano::Event::MousePressed(pelicano::MouseButton::Left), b, Some(Point::new(5.0, 5.0)), &mut |e| fora.push(e));
        assert!(c == pelicano::Captured::No && fora.is_empty());
        let e: Element<Teste> = program(g).into();
        let png = pelicano::capture(&e, b.size(), 1.0, &Theme::light(), &mut Renderer::new(), None);
        assert!(!png.is_empty());
    }

    #[test]
    fn medidores_usam_percentuais_reais_e_toleram_medida_indisponivel() {
        assert_eq!(percentual_rss(2_048, Some(102_400)), Some(2.0));
        assert_eq!(percentual_rss(2_048, Some(0)), None);
        assert_eq!(percentual_rss(0, Some(102_400)), None);
        let svg = rosca_svg(Some(25.0), "#18a64f");
        assert!(svg.contains("M 50 12 A 38 38 0 0 1 88.000 50.000"));
        assert!(!svg.contains("stroke-dasharray"));
        assert!(!svg.contains("pathLength"));
        let sem_medida = rosca_svg(None, "#18a64f");
        assert_eq!(sem_medida.matches("<circle").count(), 1);
        assert!(!sem_medida.contains("<path"));
        assert_eq!(rosca_svg(Some(0.0), "#18a64f").matches("<circle").count(), 1);
        let cheia = rosca_svg(Some(100.0), "#18a64f");
        assert_eq!(cheia.matches("<circle").count(), 2);
        assert!(!cheia.contains("<path"));
    }

    #[test]
    fn rosca_cpu_baixa_nao_repete_pontos_ao_redor_do_anel() {
        let um_porcento = rosca_svg(Some(1.2), "#8051d9");
        assert_eq!(um_porcento.matches("<circle").count(), 1);
        assert_eq!(um_porcento.matches("<path").count(), 1);
        assert!(um_porcento.contains("stroke-linecap=\"butt\""));
        let setenta_e_cinco = rosca_svg(Some(75.0), "#8051d9");
        assert!(setenta_e_cinco.contains("M 50 12 A 38 38 0 1 1 12.000 50.000"));
        for valor in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let svg = rosca_svg(Some(valor), "#8051d9");
            assert_eq!(svg.matches("<circle").count(), 1);
            assert!(!svg.contains("NaN"));
        }
        assert_eq!(rosca_svg(Some(-9.0), "#8051d9").matches("<circle").count(), 1);
        assert_eq!(rosca_svg(Some(109.0), "#8051d9").matches("<circle").count(), 2);
    }

    #[test]
    fn periodo_20_40_80_e_escala_em_mib() {
        let mut m = Monitor::novo("Teste");
        for n in [20, 40, 80] {
            let _: Task<Teste> = m.atualizar(Evento::AlternarAmostras);
            assert!(m.menu_amostras_aberto());
            let _: Task<Teste> = m.atualizar(Evento::Amostras(n));
            assert_eq!(m.amostras(), n);
            assert!(!m.menu_amostras_aberto());
        }
        let _: Task<Teste> = m.atualizar(Evento::Amostras(1000));
        assert_eq!(m.amostras(), 80);
    }

    #[test]
    fn usa_so_icones_do_catalogo() {
        for nome in ["device-desktop", "terminal", "chart-bar", "chart-line",
            "settings", "info-circle", "chevron-down", "circle-check"] {
            assert!(crate::icones::existe(nome), "Sem SVG: {nome}");
        }
    }

    #[test]
    fn console_limita_memoria_preserva_utf8_e_filtra_copia() {
        let mut c = Console::novo("Teste");
        for _ in 0..2_000 {
            c.registrar(Nivel::Info, "Teste", &"á".repeat(4_000));
        }
        assert!(c.bytes <= MAX_BYTES);
        assert!(c.registros.len() <= MAX_REGISTROS);
        assert!(c.descartados > 0);
        c.registrar(Nivel::Erro, "Arquivo", "Falha ao abrir orçamento.xlsx");
        c.filtro = Some(Nivel::Erro);
        c.busca = "ORÇAMENTO".into();
        assert_eq!(c.visiveis().count(), 1);
        assert!(c.texto_visivel().contains("[ERRO] [Arquivo]"));
        assert!(!c.texto_visivel().contains("[Teste]"));
        c.registrar_saida("Macros", "primeira linha\nErro: execução falhou");
        assert_eq!(c.registros.back().unwrap().nivel, Nivel::Erro);
    }
}
