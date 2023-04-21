import React, {
  useState, useContext, useEffect, useCallback, useRef,
} from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  Accordion,
  AccordionDetails,
  AccordionSummary,
  Alert,
  AlertColor,
  Button,
  Chip,
  Container,
  Grid,
  Paper,
  Pagination,
  Skeleton,
  Stack,
  Tabs,
  Tab,
  TextField,
  Typography,
} from '@mui/material';

import ExpandMoreIcon from '@mui/icons-material/ExpandMore';

import DetailInfoMd from 'bundle-text:../help-texts/detail-info.md';
import Markdown from './markdown';

import GraphG6, { GraphProps } from './graph-g6';
import LoadingContext from './loading-context';
import SnackbarContext from './snackbar-context';

export type Parsing = 'Naive' | 'Hybrid';

export type StrategySnakeCase = 'parse_only' | 'ground' | 'complete' | 'stable' | 'stable_counting_a' | 'stable_counting_b' | 'stable_nogood';

export type StrategyCamelCase = 'ParseOnly' | 'Ground' | 'Complete' | 'Stable' | 'StableCountingA' | 'StableCountingB' | 'StableNogood';
export const STRATEGIES_WITHOUT_PARSE: StrategyCamelCase[] = ['Ground', 'Complete', 'Stable', 'StableCountingA', 'StableCountingB', 'StableNogood'];

export interface AcAndGraph {
  ac: string[],
  graph: GraphProps,
}

export type AcsWithGraphsOpt = {
  type: 'None',
} | {
  type: 'Error',
  content: string
} | {
  type: 'Some',
  content: AcAndGraph[]
};

export type Task = {
  type: 'Parse',
} | {
  type: 'Solve',
  content: StrategyCamelCase,
};

export interface AdfProblemInfo {
  name: string,
  code: string,
  parsing_used: Parsing,
  // NOTE: the keys are really only strategies
  acs_per_strategy: { [key in StrategySnakeCase]: AcsWithGraphsOpt },
  running_tasks: Task[],
}

export function acsWithGraphOptToColor(status: AcsWithGraphsOpt, running: boolean): AlertColor {
  if (running) {
    return 'warning';
  }

  switch (status.type) {
    case 'None': return 'info';
    case 'Error': return 'error';
    case 'Some': return 'success';
    default:
      throw new Error('Unknown type union variant (cannot occur)');
  }
}

export function acsWithGraphOptToText(status: AcsWithGraphsOpt, running: boolean): string {
  if (running) {
    return 'Running';
  }

  switch (status.type) {
    case 'None': return 'Not attempted';
    case 'Error': return 'Failed';
    case 'Some': return 'Done';
    default:
      throw new Error('Unknown type union variant (cannot occur)');
  }
}

function AdfDetails() {
  const { adfName } = useParams();
  const navigate = useNavigate();

  const { setLoading } = useContext(LoadingContext);
  const { status: snackbarInfo, setStatus: setSnackbarInfo } = useContext(SnackbarContext);
  const [problem, setProblem] = useState<AdfProblemInfo>();
  const [tab, setTab] = useState<StrategySnakeCase>('parse_only');
  const [solutionIndex, setSolutionIndex] = useState<number>(0);

  const isFirstRender = useRef(true);

  const fetchProblem = useCallback(
    () => {
      fetch(`${process.env.NODE_ENV === 'development' ? '//localhost:8080' : ''}/adf/${adfName}`, {
        method: 'GET',
        credentials: process.env.NODE_ENV === 'development' ? 'include' : 'same-origin',
        headers: {
          'Content-Type': 'application/json',
        },
      })
        .then((res) => {
          switch (res.status) {
            case 200:
              res.json().then((resProblem) => {
                setProblem(resProblem);
              });
              break;
            default:
              navigate('/');
              break;
          }
        });
    },
    [setProblem],
  );

  const solveHandler = useCallback(
    (strategy: StrategyCamelCase) => {
      setLoading(true);

      fetch(`${process.env.NODE_ENV === 'development' ? '//localhost:8080' : ''}/adf/${adfName}/solve`, {
        method: 'PUT',
        credentials: process.env.NODE_ENV === 'development' ? 'include' : 'same-origin',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ strategy }),
      })
        .then((res) => {
          switch (res.status) {
            case 200:
              setSnackbarInfo({ message: 'Solving problem now...', severity: 'success', potentialUserChange: false });
              fetchProblem();
              break;
            default:
              setSnackbarInfo({ message: 'Something went wrong tying to solve the problem.', severity: 'error', potentialUserChange: false });
              break;
          }
        })
        .finally(() => setLoading(false));
    },
    [adfName],
  );

  const deleteHandler = useCallback(
    () => {
      setLoading(true);

      fetch(`${process.env.NODE_ENV === 'development' ? '//localhost:8080' : ''}/adf/${adfName}`, {
        method: 'DELETE',
        credentials: process.env.NODE_ENV === 'development' ? 'include' : 'same-origin',
        headers: {
          'Content-Type': 'application/json',
        },
      })
        .then((res) => {
          switch (res.status) {
            case 200:
              setSnackbarInfo({ message: 'ADF Problem deleted.', severity: 'success', potentialUserChange: false });
              navigate('/');
              break;
            default:
              break;
          }
        })
        .finally(() => setLoading(false));
    },
    [adfName],
  );

  useEffect(
    () => {
      // TODO: having the info if the user may have changed on the snackbar info
      // is a bit lazy and unclean; be better!
      if (isFirstRender.current || snackbarInfo?.potentialUserChange) {
        isFirstRender.current = false;

        fetchProblem();
      }
    },
    [snackbarInfo?.potentialUserChange],
  );

  useEffect(
    () => {
      // if there is a running task, fetch problems again after 20 seconds
      let timeout: ReturnType<typeof setTimeout>;
      if (problem && problem.running_tasks.length > 0) {
        timeout = setTimeout(() => fetchProblem(), 20000);
      }

      return () => {
        if (timeout) {
          clearTimeout(timeout);
        }
      };
    },
    [problem],
  );

  const acsOpt = problem?.acs_per_strategy[tab];
  const acsContent = acsOpt?.type === 'Some' ? acsOpt.content : undefined;
  const tabCamelCase: StrategyCamelCase = tab.replace(/^([a-z])/, (_, p1) => p1.toUpperCase()).replace(/_([a-z])/g, (_, p1) => `${p1.toUpperCase()}`) as StrategyCamelCase;

  return (
    <>
      <Typography variant="h3" component="h1" align="center" gutterBottom>
        ADF-BDD.DEV
      </Typography>
      <Container sx={{ marginTop: 2, marginBottom: 2 }}>
        <Accordion>
          <AccordionSummary expandIcon={<ExpandMoreIcon />}>
            <span style={{ fontWeight: 'bold' }}>What can I do with the ADF now?</span>
          </AccordionSummary>
          <AccordionDetails>
            <Grid container alignItems="center" spacing={2}>
              <Grid item xs={12} sm={8}>
                <Markdown>{DetailInfoMd}</Markdown>
              </Grid>
              <Grid item xs={12} sm={4}>
                <img
                  src={new URL('../help-texts/example-bdd.png', import.meta.url).toString()}
                  alt="Example BDD"
                  style={{ maxWidth: '100%', borderRadius: 4, boxShadow: '0 0 5px 0 rgba(0,0,0,0.4)' }}
                />
              </Grid>
            </Grid>
          </AccordionDetails>
        </Accordion>
      </Container>
      <Container sx={{ marginBottom: 4 }}>
        {problem ? (
          <>
            <Paper elevation={8} sx={{ padding: 2, marginBottom: 2 }}>
              <Stack direction="row" justifyContent="space-between" sx={{ marginBottom: 1 }}>
                <Button
                  variant="outlined"
                  color="info"
                  onClick={() => { navigate('/'); }}
                >
                  Back
                </Button>
                <Typography variant="h4" component="h2" align="center" gutterBottom>
                  {problem.name}
                </Typography>
                <Button
                  type="button"
                  variant="outlined"
                  color="error"
                  onClick={() => {
                  // eslint-disable-next-line no-alert
                    if (window.confirm('Are you sure that you want to delete this ADF problem?')) {
                      deleteHandler();
                    }
                  }}
                >
                  Delete
                </Button>
              </Stack>
              <TextField
                name="code"
                label="Code"
                helperText="Click here to copy!"
                multiline
                maxRows={5}
                fullWidth
                variant="filled"
                value={problem.code.trim()}
                disabled
                sx={{ cursor: 'pointer' }}
                onClick={() => { navigator.clipboard.writeText(problem.code); setSnackbarInfo({ message: 'Code copied to clipboard!', severity: 'info', potentialUserChange: false }); }}
              />
            </Paper>
            <Tabs
              value={tab}
              onChange={(_e, newTab) => { setTab(newTab); setSolutionIndex(0); }}
              variant="scrollable"
              scrollButtons="auto"
            >
              <Tab wrapped value="parse_only" label={<Chip color={acsWithGraphOptToColor(problem.acs_per_strategy.parse_only, problem.running_tasks.some((t: Task) => t.type === 'Parse'))} label={`${problem.parsing_used} Parsing`} sx={{ cursor: 'inherit' }} />} />
              {STRATEGIES_WITHOUT_PARSE.map((strategy) => {
                const spaced = strategy.replace(/([A-Za-z])([A-Z])/g, '$1 $2');
                const snakeCase = strategy.replace(/^([A-Z])/, (_, p1) => p1.toLowerCase()).replace(/([A-Z])/g, (_, p1) => `_${p1.toLowerCase()}`) as StrategySnakeCase;
                const status = problem.acs_per_strategy[snakeCase];

                const running = problem.running_tasks.some((t: Task) => t.type === 'Solve' && t.content === strategy);

                const color = acsWithGraphOptToColor(status, running);

                return <Tab key={strategy} wrapped value={snakeCase} label={<Chip color={color} label={spaced} sx={{ cursor: 'inherit' }} />} />;
              })}
            </Tabs>

            {acsContent && acsContent.length > 1 && (
              <>
                Models:
                <br />
                <Pagination variant="outlined" shape="rounded" count={acsContent.length} page={solutionIndex + 1} onChange={(_e, newIdx) => setSolutionIndex(newIdx - 1)} />
              </>
            )}
            <Paper elevation={3} square sx={{ padding: 2, marginTop: 4, marginBottom: 4 }}>
              {problem.running_tasks.some((t: Task) => (tab === 'parse_only' && t.type === 'Parse') || (t.type === 'Solve' && t.content === tabCamelCase)) ? (
                <Alert severity="warning">Working hard to solve the problem right now...</Alert>
              ) : (
                <>
                  {acsContent && acsContent.length > 0 && (
                  <GraphG6 graph={acsContent[solutionIndex].graph} />
                  )}
                  {acsContent && acsContent.length === 0 && (
                  <Alert severity="info">The problem has no models for this strategy.</Alert>
                  )}
                  {!acsContent && acsOpt?.type === 'Error' && (
                  <Alert severity="error">
                    An error occurred:
                    {acsOpt.content}
                  </Alert>
                  )}
                  {!acsContent && acsOpt?.type === 'None' && (
                  <>
                    <Alert severity="info" sx={{ marginBottom: 1 }}>This strategy was not attempted yet.</Alert>
                    <Button
                      variant="contained"
                      size="large"
                      color="warning"
                      onClick={() => {
                        solveHandler(tabCamelCase);
                      }}
                    >
                      Solve now!
                    </Button>
                  </>
                  )}
                </>
              )}
            </Paper>
          </>
        ) : (
          <>
            <Paper elevation={8} sx={{ padding: 2, marginBottom: 8 }}>
              <Skeleton variant="text" width="50%" sx={{ fontSize: '2.125rem', margin: 'auto' }} />
              <Skeleton variant="rounded" width="100%" height={200} />
            </Paper>
            <Skeleton variant="rectangular" width="100%" height={500} />
          </>
        )}
      </Container>
    </>
  );
}

export default AdfDetails;
