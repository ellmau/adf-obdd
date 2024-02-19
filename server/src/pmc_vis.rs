use actix_identity::Identity;
use actix_web::{get, web, HttpResponse, Responder, http::header};
use serde::Deserialize;
use crate::config::AppState;


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

const DUMMY_OUTGOING : &str = r#"{
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

#[get("/{problem_id}/initial")]
async fn pmc_vis_get_initial(
    _app_state: web::Data<AppState>,
    _identity: Option<Identity>,
    path: web::Path<String>,
) -> impl Responder {
    let _problem_id = path.into_inner();

    HttpResponse::Ok()
        .append_header(header::ContentType::json())
        .body(DUMMY_INITIAL)
    // HttpResponse::Ok().json(...)
}

#[derive(Deserialize)]
struct OutgoingQuery {
    id: String,
}

#[get("/{problem_id}/outgoing")]
async fn pmc_vis_get_outgoing(
    _app_state: web::Data<AppState>,
    _identity: Option<Identity>,
    path: web::Path<String>,
    query: web::Query<OutgoingQuery>,
) -> impl Responder {
    let _problem_id = path.into_inner();
    let _node_id = &query.id;

    HttpResponse::Ok()
        .append_header(header::ContentType::json())
        .body(DUMMY_OUTGOING)
    // HttpResponse::Ok().json(...)
}
