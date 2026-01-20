use chrono::{DateTime, Datelike, Duration, Local, NaiveDate};
use console_menu::{Menu, MenuOption, MenuProps};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PontoApp {
    horario_inicio: Option<String>,
    horario_fim: Option<String>,
    almoco_inicio: Option<String>,
    almoco_fim: Option<String>,
    data: Option<String>,
    total_horas: Option<f32>,
}

impl PontoApp {
    fn new() -> Self {
        PontoApp {
            horario_inicio: None,
            horario_fim: None,
            almoco_inicio: None,
            almoco_fim: None,
            data: None,
            total_horas: None,
        }
    }

    fn data_file_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".ponto_records.json")
    }

    fn load_records() -> Vec<PontoApp> {
        let path = Self::data_file_path();
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn save_records(records: &[PontoApp]) {
        let path = Self::data_file_path();
        if let Ok(content) = serde_json::to_string_pretty(records) {
            let _ = fs::write(&path, content);
        }
    }

    fn load_today() -> Option<PontoApp> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        Self::load_records()
            .into_iter()
            .find(|r| r.data.as_ref() == Some(&today))
    }

    fn save(&self) {
        let mut records = Self::load_records();
        let today = Local::now().format("%Y-%m-%d").to_string();

        if let Some(pos) = records.iter().position(|r| r.data.as_ref() == Some(&today)) {
            records[pos] = self.clone();
        } else {
            records.push(self.clone());
        }

        Self::save_records(&records);
    }

    fn iniciar_horario(&mut self) {
        let now = Local::now();
        self.data = Some(now.format("%Y-%m-%d").to_string());
        self.horario_inicio = Some(now.format("%H:%M:%S").to_string());
        self.save();
        println!("\n✓ Horário iniciado às {}", now.format("%H:%M:%S"));
        Self::pause();
    }

    fn iniciar_almoco(&mut self) {
        if self.horario_inicio.is_none() {
            println!("\n✗ Você precisa iniciar o horário primeiro!");
            Self::pause();
            return;
        }
        let now = Local::now();
        self.almoco_inicio = Some(now.format("%H:%M:%S").to_string());
        self.save();
        println!("\n✓ Almoço iniciado às {}", now.format("%H:%M:%S"));
        Self::pause();
    }

    fn finalizar_almoco(&mut self) {
        if self.almoco_inicio.is_none() {
            println!("\n✗ Você precisa iniciar o almoço primeiro!");
            Self::pause();
            return;
        }
        let now = Local::now();
        self.almoco_fim = Some(now.format("%H:%M:%S").to_string());
        self.save();
        println!("\n✓ Almoço finalizado às {}", now.format("%H:%M:%S"));
        Self::pause();
    }

    fn finalizar_horario(&mut self) {
        if self.horario_inicio.is_none() {
            println!("\n✗ Você precisa iniciar o horário primeiro!");
            Self::pause();
            return;
        }
        let now = Local::now();
        self.horario_fim = Some(now.format("%H:%M:%S").to_string());
        self.calcular_total();
        self.save();
        println!("\n✓ Horário finalizado às {}", now.format("%H:%M:%S"));
        if let Some(total) = self.total_horas {
            let hours = total as u32;
            let minutes = ((total - hours as f32) * 60.0) as u32;
            println!("  Total trabalhado: {}h{}m", hours, minutes);
        }
        Self::pause();
    }

    fn calcular_total(&mut self) {
        if let (Some(inicio), Some(fim)) = (&self.horario_inicio, &self.horario_fim) {
            let today = Local::now().date_naive();
            let inicio_dt = Self::parse_time(&today, inicio);
            let fim_dt = Self::parse_time(&today, fim);

            if let (Some(inicio_dt), Some(fim_dt)) = (inicio_dt, fim_dt) {
                let mut duracao = fim_dt.signed_duration_since(inicio_dt);

                // Descontar tempo de almoço
                if let (Some(almoco_ini), Some(almoco_fim)) =
                    (&self.almoco_inicio, &self.almoco_fim)
                {
                    let almoco_ini_dt = Self::parse_time(&today, almoco_ini);
                    let almoco_fim_dt = Self::parse_time(&today, almoco_fim);

                    if let (Some(almoco_ini_dt), Some(almoco_fim_dt)) =
                        (almoco_ini_dt, almoco_fim_dt)
                    {
                        let almoco_duracao = almoco_fim_dt.signed_duration_since(almoco_ini_dt);
                        duracao = duracao - almoco_duracao;
                    }
                }

                self.total_horas = Some(duracao.num_minutes() as f32 / 60.0);
            }
        }
    }

    fn parse_time(date: &NaiveDate, time_str: &str) -> Option<DateTime<Local>> {
        let datetime_str = format!("{} {}", date, time_str);
        chrono::NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|dt| dt.and_local_timezone(Local).unwrap())
    }

    fn ver_horas_dia(&self) {
        println!("\n═══════════════════════════════════════");
        println!("         HORAS DO DIA");
        println!("═══════════════════════════════════════");

        if let Some(data) = &self.data {
            println!("Data: {}", data);
        } else {
            println!("Nenhum registro para hoje.");
            Self::pause();
            return;
        }

        if let Some(inicio) = &self.horario_inicio {
            println!("Início: {}", inicio);
        }
        if let Some(almoco_ini) = &self.almoco_inicio {
            println!("Almoço início: {}", almoco_ini);
        }
        if let Some(almoco_fim) = &self.almoco_fim {
            println!("Almoço fim: {}", almoco_fim);
        }
        if let Some(fim) = &self.horario_fim {
            println!("Fim: {}", fim);
        }
        if let Some(total) = self.total_horas {
            let hours = total as u32;
            let minutes = ((total - hours as f32) * 60.0) as u32;
            println!("Total: {}h{}m", hours, minutes);
        }
        println!("═══════════════════════════════════════");
        Self::pause();
    }

    fn relatorio_diario() {
        let records = Self::load_records();
        let today = Local::now().format("%Y-%m-%d").to_string();

        println!("\n═══════════════════════════════════════");
        println!("       RELATÓRIO DIÁRIO - {}", today);
        println!("═══════════════════════════════════════");

        if let Some(record) = records.iter().find(|r| r.data.as_ref() == Some(&today)) {
            Self::print_record(record);
        } else {
            println!("Nenhum registro encontrado para hoje.");
        }
        println!("═══════════════════════════════════════");
        Self::pause();
    }

    fn relatorio_semanal() {
        let records = Self::load_records();
        let today = Local::now().date_naive();
        let week_start = today - Duration::days(today.weekday().num_days_from_monday() as i64);

        println!("\n═══════════════════════════════════════");
        println!("         RELATÓRIO SEMANAL");
        println!("═══════════════════════════════════════");

        let mut total_semana: f32 = 0.0;
        let mut dias_trabalhados = 0;

        for i in 0..7 {
            let date = week_start + Duration::days(i);
            let date_str = date.format("%Y-%m-%d").to_string();

            if let Some(record) = records.iter().find(|r| r.data.as_ref() == Some(&date_str)) {
                Self::print_record(record);
                if let Some(horas) = record.total_horas {
                    total_semana += horas;
                    dias_trabalhados += 1;
                }
                println!("---");
            }
        }

        let hours = total_semana as u32;
        let minutes = ((total_semana - hours as f32) * 60.0) as u32;
        println!("\nTotal da semana: {}h{}m", hours, minutes);
        println!("Dias trabalhados: {}", dias_trabalhados);
        println!("═══════════════════════════════════════");
        Self::pause();
    }

    fn relatorio_mensal() {
        let records = Self::load_records();
        let today = Local::now();
        let month = today.format("%Y-%m").to_string();

        println!("\n═══════════════════════════════════════");
        println!("    RELATÓRIO MENSAL - {}", today.format("%B %Y"));
        println!("═══════════════════════════════════════");

        let mut total_mes: f32 = 0.0;
        let mut dias_trabalhados = 0;

        for record in &records {
            if let Some(data) = &record.data {
                if data.starts_with(&month) {
                    Self::print_record(record);
                    if let Some(horas) = record.total_horas {
                        total_mes += horas;
                        dias_trabalhados += 1;
                    }
                    println!("---");
                }
            }
        }

        let hours = total_mes as u32;
        let minutes = ((total_mes - hours as f32) * 60.0) as u32;
        println!("\nTotal do mês: {}h{}m", hours, minutes);
        println!("Dias trabalhados: {}", dias_trabalhados);
        println!("═══════════════════════════════════════");
        Self::pause();
    }

    fn print_record(record: &PontoApp) {
        if let Some(data) = &record.data {
            println!("Data: {}", data);
        }
        if let Some(inicio) = &record.horario_inicio {
            print!("  {} - ", inicio);
        }
        if let Some(fim) = &record.horario_fim {
            print!("{}", fim);
        }
        if let Some(total) = record.total_horas {
            let hours = total as u32;
            let minutes = ((total - hours as f32) * 60.0) as u32;
            println!(" ({}h{}m)", hours, minutes);
        } else {
            println!(" (em andamento)");
        }
    }

    fn pause() {
        print!("\nPressione ENTER para continuar...");
        let _ = io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
    }

    fn remover_registro() {
        let records = Self::load_records();

        if records.is_empty() {
            println!("\n✗ Nenhum registro encontrado para remover.");
            Self::pause();
            return;
        }

        println!("\n═══════════════════════════════════════");
        println!("       REMOVER REGISTRO DE DIA");
        println!("═══════════════════════════════════════");
        println!("\nRegistros disponíveis:\n");

        for (i, record) in records.iter().enumerate() {
            if let Some(data) = &record.data {
                let total_str = if let Some(total) = record.total_horas {
                    let hours = total as u32;
                    let minutes = ((total - hours as f32) * 60.0) as u32;
                    format!(" - {}h{}m", hours, minutes)
                } else {
                    " - (em andamento)".to_string()
                };
                println!("  [{}] {}{}", i + 1, data, total_str);
            }
        }

        println!("\n  [0] Cancelar");
        println!("\n═══════════════════════════════════════");

        print!("\nDigite o número do registro para remover: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("\n✗ Erro ao ler entrada.");
            Self::pause();
            return;
        }

        let choice: usize = match input.trim().parse() {
            Ok(n) => n,
            Err(_) => {
                println!("\n✗ Opção inválida.");
                Self::pause();
                return;
            }
        };

        if choice == 0 {
            println!("\n✓ Operação cancelada.");
            Self::pause();
            return;
        }

        if choice > records.len() {
            println!("\n✗ Opção inválida.");
            Self::pause();
            return;
        }

        let record_to_remove = &records[choice - 1];
        let data_to_remove = record_to_remove.data.clone();

        print!(
            "\nTem certeza que deseja remover o registro de {}? (s/N): ",
            data_to_remove.as_deref().unwrap_or("data desconhecida")
        );
        let _ = io::stdout().flush();

        let mut confirm = String::new();
        if io::stdin().read_line(&mut confirm).is_err() {
            println!("\n✗ Erro ao ler entrada.");
            Self::pause();
            return;
        }

        if confirm.trim().to_lowercase() == "s" {
            let new_records: Vec<PontoApp> = records
                .into_iter()
                .enumerate()
                .filter(|(i, _)| *i != choice - 1)
                .map(|(_, r)| r)
                .collect();

            Self::save_records(&new_records);
            println!(
                "\n✓ Registro de {} removido com sucesso!",
                data_to_remove.as_deref().unwrap_or("data desconhecida")
            );
        } else {
            println!("\n✓ Operação cancelada.");
        }

        Self::pause();
    }
}

fn main() {
    let app = Rc::new(RefCell::new(
        PontoApp::load_today().unwrap_or_else(PontoApp::new),
    ));

    loop {
        let app_clone1 = Rc::clone(&app);
        let app_clone2 = Rc::clone(&app);
        let app_clone3 = Rc::clone(&app);
        let app_clone4 = Rc::clone(&app);
        let app_clone5 = Rc::clone(&app);

        let menu_options = vec![
            MenuOption::new("Iniciar horário", move || {
                app_clone1.borrow_mut().iniciar_horario();
            }),
            MenuOption::new("Iniciar Almoço", move || {
                app_clone2.borrow_mut().iniciar_almoco();
            }),
            MenuOption::new("Finalizar Almoço", move || {
                app_clone3.borrow_mut().finalizar_almoco();
            }),
            MenuOption::new("Finalizar horário", move || {
                app_clone4.borrow_mut().finalizar_horario();
            }),
            MenuOption::new("Relatório Diário", || PontoApp::relatorio_diario()),
            MenuOption::new("Relatório Semanal", || PontoApp::relatorio_semanal()),
            MenuOption::new("Relatório Mensal", || PontoApp::relatorio_mensal()),
            MenuOption::new("Ver horas do dia", move || {
                app_clone5.borrow().ver_horas_dia();
            }),
            MenuOption::new("Remover Registro", || PontoApp::remover_registro()),
            MenuOption::new("Sair", || std::process::exit(0)),
        ];

        let mut menu = Menu::new(menu_options, MenuProps::default());
        menu.show();
    }
}
