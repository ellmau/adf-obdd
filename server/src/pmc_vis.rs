use crate::{
    adf::{AcAndGraph, AdfProblem, OptionWithError},
    config::{AppState, RunningInfo, Task, ADF_COLL, COMPUTE_TIME, DB_NAME, USER_COLL},
    double_labeled_graph::DoubleLabeledGraph,
};
use actix_identity::Identity;
use actix_web::{get, http::header, web, HttpResponse, Responder};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

const DUMMY_INITIAL: &str = r#"{
  "nodes": [
    {
      "id": "0",
      "name": "0;0;0;0",
      "type": "s",
      "details": {
        "Variable Values": {
          "game_sum": {
            "value": 0,
            "type": "numbers"
          },
          "phase": {
            "value": 0,
            "type": "numbers"
          },
          "rounds": {
            "value": 0,
            "type": "numbers"
          },
          "sum": {
            "value": 0,
            "type": "numbers"
          }
        },
        "Reward Structures": {
          "points": {
            "value": 0.0,
            "type": "numbers"
          },
          "rolls": {
            "value": 0.0,
            "type": "numbers"
          }
        },
        "Model Checking Results": {
          "Assurence": {
            "value": 1.0,
            "type": "numbers"
          },
          "Loser": {
            "value": 0.0,
            "type": "numbers"
          },
          "Winner": {
            "value": 0.5459603775108021,
            "type": "numbers"
          }
        },
        "Atomic Propositions": {
          "deadlock": {
            "value": false,
            "type": "boolean"
          },
          "end": {
            "value": false,
            "type": "boolean"
          },
          "init": {
            "value": true,
            "icon": true,
            "identifier": "fa-solid fa-arrow-right",
            "type": "boolean"
          },
          "lost": {
            "value": false,
            "type": "boolean"
          },
          "won": {
            "value": false,
            "type": "boolean"
          }
        }
      },
      "viewDetails": {
        "cluster identifier": null
      }
    }
  ],
  "edges": [],
  "info": {
    "Atomic Propositions": {
      "init": {
        "identifier": "fa-solid fa-arrow-right",
        "icon": true
      },
      "lost": {
        "identifier": "l0",
        "icon": false
      },
      "won": {
        "identifier": "w0",
        "icon": false
      },
      "end": {
        "identifier": "e0",
        "icon": false
      },
      "deadlock": {
        "identifier": "fa-solid fa-rotate-right",
        "icon": true
      }
    },
    "ID": "Pig2",
    "Model Checking Results": {
      "Assurence": "1.0 (exact floating point)",
      "Loser": "0.0 (exact floating point)",
      "Winner": "0.5459603775108021 (+/- 4.479762978110217E-6 estimated; rel err 8.205289545982817E-6)"
    },
    "Scheduler": {
      "Assurence": 0,
      "Loser": 2,
      "Winner": 1
    }
  },
  "scheduler": [
    "Assurence",
    "Winner",
    "Loser"
  ]
}"#;

const DUMMY_OUTGOING: &str = r#"{
  "nodes": [
    {
      "id": "0",
      "name": "0;0;0;0",
      "type": "s",
      "details": {
        "Variable Values": {
          "game_sum": {
            "value": 0,
            "type": "numbers"
          },
          "phase": {
            "value": 0,
            "type": "numbers"
          },
          "rounds": {
            "value": 0,
            "type": "numbers"
          },
          "sum": {
            "value": 0,
            "type": "numbers"
          }
        },
        "Reward Structures": {
          "points": {
            "value": 0.0,
            "type": "numbers"
          },
          "rolls": {
            "value": 0.0,
            "type": "numbers"
          }
        },
        "Model Checking Results": {
          "Assurence": {
            "value": 1.0,
            "type": "numbers"
          },
          "Loser": {
            "value": 0.0,
            "type": "numbers"
          },
          "Winner": {
            "value": 0.5459603775108021,
            "type": "numbers"
          }
        },
        "Atomic Propositions": {
          "deadlock": {
            "value": false,
            "type": "boolean"
          },
          "end": {
            "value": false,
            "type": "boolean"
          },
          "init": {
            "value": true,
            "icon": true,
            "identifier": "fa-solid fa-arrow-right",
            "type": "boolean"
          },
          "lost": {
            "value": false,
            "type": "boolean"
          },
          "won": {
            "value": false,
            "type": "boolean"
          }
        }
      },
      "viewDetails": {
        "cluster identifier": null
      }
    },
    {
      "id": "1",
      "name": "0;0;0;2",
      "type": "s",
      "details": {
        "Variable Values": {
          "game_sum": {
            "value": 0,
            "type": "numbers"
          },
          "phase": {
            "value": 2,
            "type": "numbers"
          },
          "rounds": {
            "value": 0,
            "type": "numbers"
          },
          "sum": {
            "value": 0,
            "type": "numbers"
          }
        },
        "Reward Structures": {
          "points": {
            "value": 0.0,
            "type": "numbers"
          },
          "rolls": {
            "value": 0.0,
            "type": "numbers"
          }
        },
        "Model Checking Results": {
          "Assurence": {
            "value": 1.0,
            "type": "numbers"
          },
          "Loser": {
            "value": 0.0,
            "type": "numbers"
          },
          "Winner": {
            "value": 0.4200261401421442,
            "type": "numbers"
          }
        },
        "Atomic Propositions": {
          "deadlock": {
            "value": false,
            "type": "boolean"
          },
          "end": {
            "value": false,
            "type": "boolean"
          },
          "init": {
            "value": false,
            "type": "boolean"
          },
          "lost": {
            "value": false,
            "type": "boolean"
          },
          "won": {
            "value": false,
            "type": "boolean"
          }
        }
      },
      "viewDetails": {
        "cluster identifier": null
      }
    },
    {
      "id": "3",
      "name": "0;0;2;1",
      "type": "s",
      "details": {
        "Variable Values": {
          "game_sum": {
            "value": 0,
            "type": "numbers"
          },
          "phase": {
            "value": 1,
            "type": "numbers"
          },
          "rounds": {
            "value": 0,
            "type": "numbers"
          },
          "sum": {
            "value": 2,
            "type": "numbers"
          }
        },
        "Reward Structures": {
          "points": {
            "value": 0.0,
            "type": "numbers"
          },
          "rolls": {
            "value": 0.0,
            "type": "numbers"
          }
        },
        "Model Checking Results": {
          "Assurence": {
            "value": 1.0,
            "type": "numbers"
          },
          "Loser": {
            "value": 0.0,
            "type": "numbers"
          },
          "Winner": {
            "value": 0.5578419259350655,
            "type": "numbers"
          }
        },
        "Atomic Propositions": {
          "deadlock": {
            "value": false,
            "type": "boolean"
          },
          "end": {
            "value": false,
            "type": "boolean"
          },
          "init": {
            "value": false,
            "type": "boolean"
          },
          "lost": {
            "value": false,
            "type": "boolean"
          },
          "won": {
            "value": false,
            "type": "boolean"
          }
        }
      },
      "viewDetails": {
        "cluster identifier": null
      }
    },
    {
      "id": "6",
      "name": "0;0;3;1",
      "type": "s",
      "details": {
        "Variable Values": {
          "game_sum": {
            "value": 0,
            "type": "numbers"
          },
          "phase": {
            "value": 1,
            "type": "numbers"
          },
          "rounds": {
            "value": 0,
            "type": "numbers"
          },
          "sum": {
            "value": 3,
            "type": "numbers"
          }
        },
        "Reward Structures": {
          "points": {
            "value": 0.0,
            "type": "numbers"
          },
          "rolls": {
            "value": 0.0,
            "type": "numbers"
          }
        },
        "Model Checking Results": {
          "Assurence": {
            "value": 1.0,
            "type": "numbers"
          },
          "Loser": {
            "value": 0.0,
            "type": "numbers"
          },
          "Winner": {
            "value": 0.5641941966513407,
            "type": "numbers"
          }
        },
        "Atomic Propositions": {
          "deadlock": {
            "value": false,
            "type": "boolean"
          },
          "end": {
            "value": false,
            "type": "boolean"
          },
          "init": {
            "value": false,
            "type": "boolean"
          },
          "lost": {
            "value": false,
            "type": "boolean"
          },
          "won": {
            "value": false,
            "type": "boolean"
          }
        }
      },
      "viewDetails": {
        "cluster identifier": null
      }
    },
    {
      "id": "9",
      "name": "0;0;4;1",
      "type": "s",
      "details": {
        "Variable Values": {
          "game_sum": {
            "value": 0,
            "type": "numbers"
          },
          "phase": {
            "value": 1,
            "type": "numbers"
          },
          "rounds": {
            "value": 0,
            "type": "numbers"
          },
          "sum": {
            "value": 4,
            "type": "numbers"
          }
        },
        "Reward Structures": {
          "points": {
            "value": 0.0,
            "type": "numbers"
          },
          "rolls": {
            "value": 0.0,
            "type": "numbers"
          }
        },
        "Model Checking Results": {
          "Assurence": {
            "value": 1.0,
            "type": "numbers"
          },
          "Loser": {
            "value": 0.0,
            "type": "numbers"
          },
          "Winner": {
            "value": 0.5708391005779379,
            "type": "numbers"
          }
        },
        "Atomic Propositions": {
          "deadlock": {
            "value": false,
            "type": "boolean"
          },
          "end": {
            "value": false,
            "type": "boolean"
          },
          "init": {
            "value": false,
            "type": "boolean"
          },
          "lost": {
            "value": false,
            "type": "boolean"
          },
          "won": {
            "value": false,
            "type": "boolean"
          }
        }
      },
      "viewDetails": {
        "cluster identifier": null
      }
    },
    {
      "id": "12",
      "name": "0;0;5;1",
      "type": "s",
      "details": {
        "Variable Values": {
          "game_sum": {
            "value": 0,
            "type": "numbers"
          },
          "phase": {
            "value": 1,
            "type": "numbers"
          },
          "rounds": {
            "value": 0,
            "type": "numbers"
          },
          "sum": {
            "value": 5,
            "type": "numbers"
          }
        },
        "Reward Structures": {
          "points": {
            "value": 0.0,
            "type": "numbers"
          },
          "rolls": {
            "value": 0.0,
            "type": "numbers"
          }
        },
        "Model Checking Results": {
          "Assurence": {
            "value": 1.0,
            "type": "numbers"
          },
          "Loser": {
            "value": 0.0,
            "type": "numbers"
          },
          "Winner": {
            "value": 0.577792783058856,
            "type": "numbers"
          }
        },
        "Atomic Propositions": {
          "deadlock": {
            "value": false,
            "type": "boolean"
          },
          "end": {
            "value": false,
            "type": "boolean"
          },
          "init": {
            "value": false,
            "type": "boolean"
          },
          "lost": {
            "value": false,
            "type": "boolean"
          },
          "won": {
            "value": false,
            "type": "boolean"
          }
        }
      },
      "viewDetails": {
        "cluster identifier": null
      }
    },
    {
      "id": "15",
      "name": "0;0;6;1",
      "type": "s",
      "details": {
        "Variable Values": {
          "game_sum": {
            "value": 0,
            "type": "numbers"
          },
          "phase": {
            "value": 1,
            "type": "numbers"
          },
          "rounds": {
            "value": 0,
            "type": "numbers"
          },
          "sum": {
            "value": 6,
            "type": "numbers"
          }
        },
        "Reward Structures": {
          "points": {
            "value": 0.0,
            "type": "numbers"
          },
          "rolls": {
            "value": 0.0,
            "type": "numbers"
          }
        },
        "Model Checking Results": {
          "Assurence": {
            "value": 1.0,
            "type": "numbers"
          },
          "Loser": {
            "value": 0.0,
            "type": "numbers"
          },
          "Winner": {
            "value": 0.5850696982205607,
            "type": "numbers"
          }
        },
        "Atomic Propositions": {
          "deadlock": {
            "value": false,
            "type": "boolean"
          },
          "end": {
            "value": false,
            "type": "boolean"
          },
          "init": {
            "value": false,
            "type": "boolean"
          },
          "lost": {
            "value": false,
            "type": "boolean"
          },
          "won": {
            "value": false,
            "type": "boolean"
          }
        }
      },
      "viewDetails": {
        "cluster identifier": null
      }
    },
    {
      "id": "t_6624",
      "scheduler": {
        "Assurence": 1.0,
        "Loser": 1.0,
        "Winner": 1.0
      },
      "name": null,
      "type": "t",
      "details": {
        "Scheduler": {
          "Assurence": {
            "value": 1.0,
            "type": "numbers"
          },
          "Loser": {
            "value": 1.0,
            "type": "numbers"
          },
          "Winner": {
            "value": 1.0,
            "type": "numbers"
          }
        },
        "Variable Values": {
          "action": {
            "value": "[roll]",
            "type": "numbers"
          },
          "origin": {
            "value": "0",
            "type": "numbers"
          },
          "outcome distribution": {
            "value": {
              "1": 0.16666666666666666,
              "12": 0.16666666666666666,
              "3": 0.16666666666666666,
              "15": 0.16666666666666666,
              "6": 0.16666666666666666,
              "9": 0.16666666666666666
            },
            "type": "numbers"
          }
        },
        "Reward Structures": {
          "points": {
            "value": 0.0,
            "type": "numbers"
          },
          "rolls": {
            "value": 0.0,
            "type": "numbers"
          }
        },
        "Model Checking Results": {
          "Assurence": {
            "value": 0.9999999999999999,
            "type": "numbers"
          },
          "Loser": {
            "value": 0.0,
            "type": "numbers"
          },
          "Winner": {
            "value": 0.5459606407643175,
            "type": "numbers"
          }
        }
      },
      "viewDetails": {
        "cluster identifier": []
      }
    }
  ],
  "edges": [
    {
      "source": "0",
      "target": "t_6624",
      "label": "[roll]"
    },
    {
      "source": "t_6624",
      "target": "1",
      "label": "0.16666666666666666"
    },
    {
      "source": "t_6624",
      "target": "12",
      "label": "0.16666666666666666"
    },
    {
      "source": "t_6624",
      "target": "3",
      "label": "0.16666666666666666"
    },
    {
      "source": "t_6624",
      "target": "15",
      "label": "0.16666666666666666"
    },
    {
      "source": "t_6624",
      "target": "6",
      "label": "0.16666666666666666"
    },
    {
      "source": "t_6624",
      "target": "9",
      "label": "0.16666666666666666"
    }
  ],
  "info": {
    "Atomic Propositions": {
      "init": {
        "identifier": "fa-solid fa-arrow-right",
        "icon": true
      },
      "lost": {
        "identifier": "l0",
        "icon": false
      },
      "won": {
        "identifier": "w0",
        "icon": false
      },
      "end": {
        "identifier": "e0",
        "icon": false
      },
      "deadlock": {
        "identifier": "fa-solid fa-rotate-right",
        "icon": true
      }
    },
    "ID": "Pig2",
    "Model Checking Results": {
      "Assurence": "1.0 (exact floating point)",
      "Loser": "0.0 (exact floating point)",
      "Winner": "0.5459603775108021 (+/- 4.479762978110217E-6 estimated; rel err 8.205289545982817E-6)"
    },
    "Scheduler": {
      "Assurence": 0,
      "Loser": 2,
      "Winner": 1
    }
  },
  "scheduler": [
    "Assurence",
    "Winner",
    "Loser"
  ]
}"#;

#[derive(Serialize)]
struct PmcVisNode {
    id: String,
    name: String,
    #[serde(rename = "type")]
    node_type: String,
}

#[derive(Serialize)]
struct PmcVisEdge {
    source: String,
    target: String,
    label: String,
}

struct PmcVisGraph {
    nodes: Vec<PmcVisNode>,
    edges: Vec<PmcVisEdge>,
}

impl From<DoubleLabeledGraph> for PmcVisGraph {
    fn from(graph: DoubleLabeledGraph) -> Self {
        PmcVisGraph {
            nodes: graph
                .nodes_iter()
                .map(|(k, v)| PmcVisNode {
                    id: k,
                    name: v,
                    node_type: "s".to_string(),
                })
                .collect(),
            edges: graph
                .edges_iter()
                .map(|(s, t, l)| PmcVisEdge {
                    source: s,
                    target: t,
                    label: l,
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct PmcVisInfo {
    #[serde(rename = "ID")]
    id: String,
}

#[derive(Serialize)]
struct PmcVisDto {
    nodes: Vec<PmcVisNode>,
    edges: Vec<PmcVisEdge>,
    info: PmcVisInfo,
}

impl PmcVisDto {
    fn from_pmc_vis_graph_and_id(graph: PmcVisGraph, id: String) -> Self {
        Self {
            nodes: graph.nodes,
            edges: graph.edges,
            info: PmcVisInfo { id },
        }
    }
}

#[get("/{problem_name}/initial")]
async fn pmc_vis_get_initial(
    app_state: web::Data<AppState>,
    identity: Option<Identity>,
    path: web::Path<String>,
) -> impl Responder {
    let problem_name = path.into_inner();

    let adf_coll: mongodb::Collection<AdfProblem> = app_state
        .mongodb_client
        .database(DB_NAME)
        .collection(ADF_COLL);

    let username = match identity.map(|id| id.id()) {
        None => {
            return HttpResponse::Unauthorized().body("You need to login to get an ADF problem.")
        }
        Some(Err(err)) => return HttpResponse::InternalServerError().body(err.to_string()),
        Some(Ok(username)) => username,
    };

    let adf_problem = match adf_coll
        .find_one(doc! { "name": &problem_name, "username": &username }, None)
        .await
    {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(None) => {
            return HttpResponse::NotFound()
                .body(format!("ADF problem with name {problem_name} not found."))
        }
        Ok(Some(prob)) => prob,
    };

    let parse_only_graph: DoubleLabeledGraph = match adf_problem.acs_per_strategy.parse_only {
        OptionWithError::None => {
            return HttpResponse::BadRequest().body("The ADF problem has not been parsed yet.")
        }
        OptionWithError::Error(err) => {
            return HttpResponse::BadRequest().body(format!(
                "The ADF problem could not be parsed. Update it and try again. Error: {err}"
            ))
        }
        OptionWithError::Some(acs_and_graphs) => {
            acs_and_graphs
                .first()
                .expect("There should be exacly one graph in the parsing result.")
                .clone()
                .graph
        }
    };

    HttpResponse::Ok().json(PmcVisDto::from_pmc_vis_graph_and_id(
        PmcVisGraph::from(parse_only_graph.only_roots()),
        problem_name,
    ))

    // HttpResponse::Ok()
    //     .append_header(header::ContentType::json())
    //     .body(DUMMY_INITIAL)
}

#[derive(Deserialize)]
struct OutgoingQuery {
    id: String,
}

#[get("/{problem_id}/outgoing")]
async fn pmc_vis_get_outgoing(
    app_state: web::Data<AppState>,
    identity: Option<Identity>,
    path: web::Path<String>,
    query: web::Query<OutgoingQuery>,
) -> impl Responder {
    let problem_name = path.into_inner();
    let node_id = &query.id;

    let adf_coll: mongodb::Collection<AdfProblem> = app_state
        .mongodb_client
        .database(DB_NAME)
        .collection(ADF_COLL);

    let username = match identity.map(|id| id.id()) {
        None => {
            return HttpResponse::Unauthorized().body("You need to login to get an ADF problem.")
        }
        Some(Err(err)) => return HttpResponse::InternalServerError().body(err.to_string()),
        Some(Ok(username)) => username,
    };

    let adf_problem = match adf_coll
        .find_one(doc! { "name": &problem_name, "username": &username }, None)
        .await
    {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(None) => {
            return HttpResponse::NotFound()
                .body(format!("ADF problem with name {problem_name} not found."))
        }
        Ok(Some(prob)) => prob,
    };

    let parse_only_graph: DoubleLabeledGraph = match adf_problem.acs_per_strategy.parse_only {
        OptionWithError::None => {
            return HttpResponse::BadRequest().body("The ADF problem has not been parsed yet.")
        }
        OptionWithError::Error(err) => {
            return HttpResponse::BadRequest().body(format!(
                "The ADF problem could not be parsed. Update it and try again. Error: {err}"
            ))
        }
        OptionWithError::Some(acs_and_graphs) => {
            acs_and_graphs
                .first()
                .expect("There should be exacly one graph in the parsing result.")
                .clone()
                .graph
        }
    };

    HttpResponse::Ok().json(PmcVisDto::from_pmc_vis_graph_and_id(
        PmcVisGraph::from(parse_only_graph.only_node_with_successors_roots(node_id.to_string())),
        problem_name,
    ))

    // HttpResponse::Ok()
    //     .append_header(header::ContentType::json())
    //     .body(DUMMY_OUTGOING)
}
