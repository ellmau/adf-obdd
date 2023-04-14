import React, {
  useRef, useState, useCallback, useEffect, useContext,
} from 'react';

import {
  useNavigate,
} from 'react-router-dom';

import {
  Chip,
  Container,
  Paper,
  TableContainer,
  Table,
  TableHead,
  TableRow,
  TableCell,
  TableBody,
  Typography,
} from '@mui/material';

import AdfNewForm from './adf-new-form';

import {
  AdfProblemInfo,
  StrategySnakeCase,
  STRATEGIES_WITHOUT_PARSE,
  Task,
  acsWithGraphOptToColor,
  acsWithGraphOptToText,
} from './adf-details';

import SnackbarContext from './snackbar-context';

function AdfOverview() {
  const { status: snackbarInfo } = useContext(SnackbarContext);
  const [problems, setProblems] = useState<AdfProblemInfo[]>([]);

  const navigate = useNavigate();

  const isFirstRender = useRef(true);

  const fetchProblems = useCallback(
    () => {
      fetch(`${process.env.NODE_ENV === 'development' ? '//localhost:8080' : ''}/adf/`, {
        method: 'GET',
        credentials: process.env.NODE_ENV === 'development' ? 'include' : 'same-origin',
        headers: {
          'Content-Type': 'application/json',
        },
      })
        .then((res) => {
          switch (res.status) {
            case 200:
              res.json().then((resProblems) => {
                setProblems(resProblems);
              });
              break;
            case 401:
              setProblems([]);
              break;
            default:
              break;
          }
        });
    },
    [setProblems],
  );

  useEffect(
    () => {
      // TODO: having the info if the user may have changed on the snackbar info
      // is a bit lazy and unclean; be better!
      if (isFirstRender.current || snackbarInfo?.potentialUserChange) {
        isFirstRender.current = false;

        fetchProblems();
      }
    },
    [snackbarInfo?.potentialUserChange],
  );

  useEffect(
    () => {
      // if there is a running task, fetch problems again after 20 seconds
      let timeout: ReturnType<typeof setTimeout>;
      if (problems.some((p) => p.running_tasks.length > 0)) {
        timeout = setTimeout(() => fetchProblems(), 20000);
      }

      return () => {
        if (timeout) {
          clearTimeout(timeout);
        }
      };
    },
    [problems],
  );

  return (
    <>
      <Typography variant="h3" component="h1" align="center" gutterBottom>
        ADF-BDD.DEV
      </Typography>
      {problems.length > 0
        && (
        <Container sx={{ marginBottom: 4 }}>
          <Paper elevation={8} sx={{ padding: 2 }}>
            <Typography variant="h4" component="h2" align="center" gutterBottom>
              Existing Problems
            </Typography>
            <TableContainer component={Paper}>
              <Table>
                <TableHead>
                  <TableRow>
                    <TableCell align="center">ADF Problem Name</TableCell>
                    <TableCell align="center">Parse Status</TableCell>
                    <TableCell align="center">Grounded Solution</TableCell>
                    <TableCell align="center">Complete Solution</TableCell>
                    <TableCell align="center">Stable Solution</TableCell>
                    <TableCell align="center">Stable Solution (Counting Method A)</TableCell>
                    <TableCell align="center">Stable Solution (Counting Method B)</TableCell>
                    <TableCell align="center">Stable Solution (Nogood-Based)</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {problems.map((problem) => (
                    <TableRow
                      key={problem.name}
                      onClick={() => { navigate(`/${problem.name}`); }}
                      sx={{ '&:last-child td, &:last-child th': { border: 0 }, cursor: 'pointer' }}
                    >
                      <TableCell component="th" scope="row">
                        {problem.name}
                      </TableCell>
                      {
                (() => {
                  const status = problem.acs_per_strategy.parse_only;
                  const running = problem.running_tasks.some((t: Task) => t.type === 'Parse');

                  const color = acsWithGraphOptToColor(status, running);
                  const text = acsWithGraphOptToText(status, running);

                  return <TableCell align="center"><Chip color={color} label={`${text} (${problem.parsing_used} Parsing)`} sx={{ cursor: 'inherit' }} /></TableCell>;
                })()
              }
                      {
                  STRATEGIES_WITHOUT_PARSE.map((strategy) => {
                    const status = problem.acs_per_strategy[strategy.replace(/^([A-Z])/, (_, p1) => p1.toLowerCase()).replace(/([A-Z])/g, (_, p1) => `_${p1.toLowerCase()}`) as StrategySnakeCase];
                    const running = problem.running_tasks.some((t: Task) => t.type === 'Solve' && t.content === strategy);

                    const color = acsWithGraphOptToColor(status, running);
                    const text = acsWithGraphOptToText(status, running);

                    return <TableCell key={strategy} align="center"><Chip color={color} label={text} sx={{ cursor: 'inherit' }} /></TableCell>;
                  })
              }
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          </Paper>
        </Container>
        )}
      <AdfNewForm fetchProblems={fetchProblems} />
    </>
  );
}

export default AdfOverview;
