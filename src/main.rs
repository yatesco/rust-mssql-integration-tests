use async_trait::async_trait;
use bb8::Pool;
use tiberius::Client;
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

#[derive(Clone, Debug)]
pub struct TiberiusConnectionManager {
    url: String,
}

impl TiberiusConnectionManager {
    /// Create a new `TiberiusConnectionManager`.
    pub fn new(url: String) -> tiberius::Result<TiberiusConnectionManager> {
        Ok(TiberiusConnectionManager { url })
    }
}

#[async_trait]
impl bb8::ManageConnection for TiberiusConnectionManager {
    type Connection = Client<Compat<TcpStream>>;
    type Error = tiberius::error::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        println!("URL: {}", &self.url);
        let config = tiberius::Config::from_ado_string(&self.url)?;
        println!("Creating {:?}", &config);

        let tcp = TcpStream::connect(config.get_addr()).await?;
        tcp.set_nodelay(true)?;

        Ok(Client::connect(config, tcp.compat_write()).await?)
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        println!("Checking {:?}", conn);
        conn.simple_query("").await?.into_row().await?;
        Ok(())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

#[cfg(test)]
pub struct MSSQL {
    env_vars: std::collections::HashMap<String, String>,
}

#[cfg(test)]
impl Default for MSSQL {
    fn default() -> Self {
        let mut env_vars = std::collections::HashMap::default();
        env_vars.insert("SA_PASSWORD".to_owned(), "ThePassword1!".to_owned());
        env_vars.insert("ACCEPT_EULA".into(), "Y".into());
        Self { env_vars }
    }
}

#[cfg(test)]
impl testcontainers::Image for MSSQL {
    type Args = ();

    fn name(&self) -> String {
        String::from("mcr.microsoft.com/mssql/server")
    }

    fn tag(&self) -> String {
        String::from("2017-CU12")
    }

    fn ready_conditions(&self) -> Vec<testcontainers::core::WaitFor> {
        vec![
            testcontainers::core::WaitFor::message_on_stdout("Recovery is complete. This is an informational message only. No user action is required."),
            // WaitFor::millis(2000),
        ]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item=(&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}

async fn create_connection_pool(conn_str: String) -> anyhow::Result<Pool<TiberiusConnectionManager>> {
    let mgr = TiberiusConnectionManager { url: conn_str };
    let pool = bb8::Pool::builder()
        .max_size(2)
        .build(mgr)
        .await
        .expect("cannot create pool");

    println!("Connecting in connection pool...");
    let tmp_pool = pool.clone();
    let mut conn = tmp_pool.get().await?;
    conn.simple_query("")
        .await
        .unwrap()
        .into_row()
        .await
        .unwrap();
    println!("Connected in connection pool");
    Ok(pool)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[tokio::test]
    #[serial_test::serial]
    async fn mssql_smoke_test() -> anyhow::Result<()> {
        let _ = pretty_env_logger::try_init();
        let docker = testcontainers::clients::Cli::default();

        let (_node, pool) = spin_up_database(&docker).await.expect("oops");
        println!("Connecting in test...");
        let mut conn = pool.get().await?;
        conn.simple_query("")
            .await
            .unwrap()
            .into_row()
            .await
            .unwrap();
        println!("Connected in test");
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn mssql_smoke_test_1() -> anyhow::Result<()> {
        let _ = pretty_env_logger::try_init();
        let docker = testcontainers::clients::Cli::default();

        let (_node, pool) = spin_up_database(&docker).await.expect("oops");
        println!("Connecting in test...");
        let mut conn = pool.get().await?;
        conn.simple_query("")
            .await
            .unwrap()
            .into_row()
            .await
            .unwrap();
        println!("Connected in test");
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn mssql_smoke_test_2() -> anyhow::Result<()> {
        let _ = pretty_env_logger::try_init();
        let docker = testcontainers::clients::Cli::default();

        let (_node, pool) = spin_up_database(&docker).await.expect("oops");
        println!("Connecting in test...");
        let mut conn = pool.get().await?;
        conn.simple_query("")
            .await
            .unwrap()
            .into_row()
            .await
            .unwrap();
        println!("Connected in test");
        Ok(())
    }

    pub async fn spin_up_database(
        docker: &testcontainers::clients::Cli,
    ) -> anyhow::Result<(testcontainers::Container<'_, MSSQL>, Pool<TiberiusConnectionManager>)> {
        let node = docker.run(MSSQL::default());
        let port = node.get_host_port_ipv4(1433);
        let conn_str = format!("server=tcp:127.0.0.1,{};encrypt=false;TrustServerCertificate=true;user=sa;password=ThePassword1!", port);
        println!("trying {}", &conn_str);

        let pool = create_connection_pool(conn_str).await?;
        Ok((node, pool))
    }
}

fn main() {
    println!("Hello, world!");
}
