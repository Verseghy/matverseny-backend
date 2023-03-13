use crate::{handlers::socket::Event, utils::topics};
use entity::{problems, problems_order};
use futures::{Stream, StreamExt};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    ClientConfig, Message, TopicPartitionList,
};
use sea_orm::{
    ConnectionTrait, DbConn, EntityName, EntityTrait, FromQueryResult, JoinType, QuerySelect,
    RelationTrait, TransactionTrait,
};
use std::{
    collections::HashMap,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::{
    sync::{broadcast, mpsc, RwLock},
    task,
    time::sleep,
};
use uuid::Uuid;

#[derive(Debug, FromQueryResult)]
pub struct Problem {
    pub id: Uuid,
    pub body: String,
    pub image: Option<String>,
    pub next: Option<Uuid>,
}

pub struct Problems {
    problems: Arc<RwLock<Vec<Problem>>>,
    channel: broadcast::Sender<Event>,
}

impl Problems {
    pub async fn new(db: &DbConn) -> Self {
        let txn = db.begin().await.expect("failed to start transaction");

        txn.execute_unprepared(&format!(
            r#"lock table {}, {} in share mode"#,
            problems::Entity.table_name(),
            problems_order::Entity.table_name(),
        ))
        .await
        .expect("failed to lock tables");

        let res = problems_order::Entity::find()
            .select_only()
            .column(problems::Column::Id)
            .column(problems::Column::Body)
            .column(problems::Column::Solution)
            .column(problems::Column::Image)
            .column(problems_order::Column::Next)
            .join(JoinType::InnerJoin, problems_order::Relation::Problem.def())
            .into_model::<Problem>()
            .all(&txn)
            .await
            .expect("failed to query the problems");

        let problems = Arc::new(RwLock::new(sort_initial_problems(res)));

        let bootstrap_servers = std::env::var("KAFKA_BOOTSTRAP_SERVERS")
            .expect("environment variable KAFKA_BOOTSTRAP_SERVERS is not set");

        let mut buf = [0u8; uuid::fmt::Simple::LENGTH];
        let id = uuid::Uuid::new_v4()
            .as_simple()
            .encode_lower(&mut buf);

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("group.id", id)
            .set("enable.partition.eof", "false")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "latest")
            .create()
            .expect("Failed to create kafka consumer");

        consumer
            .assign(&{
                let mut list = TopicPartitionList::new();
                list.add_partition(topics::problems(), 0);
                list
            })
            .expect("failed to assign topics");

        sleep(std::time::Duration::from_millis(200)).await;

        txn.commit().await.expect("failed to commit transaction");

        let (tx, _) = broadcast::channel(64);

        task::spawn({
            let problems = Arc::clone(&problems);
            let tx = tx.clone();
            async move {
                let mut stream = consumer.stream();
                while let Some(message) = stream.next().await {
                    let message = match message {
                        Err(err) => {
                            error!("kafka error: {err:?}");
                            break;
                        }
                        Ok(message) => message,
                    };

                    let Some(Ok(payload)) = message.payload_view::<str>() else {
                        error!("payload is not utf-8 string");
                        break
                    };

                    let event: Event = serde_json::from_str(payload).unwrap();
                    debug!("problems message: {event:?}");

                    let mut guard = problems.write().await;

                    match &event {
                        Event::InsertProblem {
                            before,
                            id,
                            body,
                            image,
                        } => {
                            let problem = Problem {
                                id: *id,
                                body: body.clone(),
                                image: image.clone(),
                                next: None,
                            };
                            if let Some(before) = before {
                                let pos = guard.iter().position(|p| p.id == *before);
                                if let Some(pos) = pos {
                                    guard.insert(pos, problem);
                                } else {
                                    warn!("no problem with id: {}", before);
                                }
                            } else {
                                guard.push(problem);
                            }
                        }
                        Event::DeleteProblem { id } => {
                            let pos = guard.iter().position(|p| p.id == *id);
                            if let Some(pos) = pos {
                                guard.remove(pos);
                            } else {
                                warn!("no problem with id: {}", id);
                            }
                        }
                        Event::SwapProblems { id1, id2 } => {
                            let pos1 = guard.iter().position(|p| p.id == *id1);
                            let pos2 = guard.iter().position(|p| p.id == *id2);

                            if let (Some(pos1), Some(pos2)) = (pos1, pos2) {
                                guard.swap(pos1, pos2);
                            } else {
                                warn!("no problems with ids: {}, {}", id1, id2);
                            }
                        }
                        Event::UpdateProblem { id, body, image } => {
                            let pos = guard.iter().position(|p| p.id == *id);

                            if let Some(pos) = pos {
                                if let Some(body) = body {
                                    guard[pos].body = body.clone();
                                }
                                if let Some(image) = image {
                                    guard[pos].image = image.clone();
                                }
                            } else {
                                warn!("no problems with id: {}", id);
                            }
                        }
                        _ => unreachable!(),
                    };

                    drop(guard);

                    let _ = tx.send(event);
                }
            }
        });

        Self {
            problems,
            channel: tx,
        }
    }

    pub async fn stream(&self) -> (ProblemStream, ProblemStream) {
        let (tx, rx) = mpsc::unbounded_channel();
        let (tx2, rx2) = mpsc::unbounded_channel();

        let guard = self.problems.read().await;

        for problem in guard.iter() {
            tx.send(Event::InsertProblem {
                before: None,
                id: problem.id,
                body: problem.body.clone(),
                image: problem.image.clone(),
            })
            .unwrap();
        }

        let mut rx3 = self.channel.subscribe();
        drop(guard);

        {
            task::spawn(async move {
                while let Ok(message) = rx3.recv().await {
                    if tx2.send(message).is_err() {
                        break;
                    }
                }
            });
        }

        (
            ProblemStream { channel: rx },
            ProblemStream { channel: rx2 },
        )
    }
}

pub struct ProblemStream {
    channel: mpsc::UnboundedReceiver<Event>,
}

impl Stream for ProblemStream {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.channel.poll_recv(cx)
    }
}

fn sort_initial_problems(problems: Vec<Problem>) -> Vec<Problem> {
    let length = problems.len();

    if length == 0 {
        return Vec::new();
    }

    let mut map = HashMap::new();

    for problem in problems {
        map.insert(problem.next, problem);
    }

    let mut result = Vec::with_capacity(length);

    let mut last_id = {
        let last = map.remove(&None).unwrap();
        let last_id = last.id;
        result.push(last);
        last_id
    };

    while !map.is_empty() {
        let item = map.remove(&Some(last_id)).unwrap();
        last_id = item.id;
        result.push(item);
    }

    debug_assert_eq!(result.len(), length);

    result.reverse();

    result
}
