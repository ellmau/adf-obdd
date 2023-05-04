import React, { useState, useMemo } from 'react';

import { createBrowserRouter, RouterProvider } from 'react-router-dom';

import { ThemeProvider, createTheme } from '@mui/material/styles';
import {
  Alert,
  AlertColor,
  Backdrop,
  Container,
  CircularProgress,
  CssBaseline,
  Link,
  Snackbar,
  Stack,
  useMediaQuery,
} from '@mui/material';

import LoadingContext from './loading-context';
import SnackbarContext from './snackbar-context';
import Footer from './footer';
import AdfOverview from './adf-overview';
import AdfDetails from './adf-details';

const browserRouter = createBrowserRouter([
  {
    path: '/',
    element: <AdfOverview />,
  },
  {
    path: '/:adfName',
    element: <AdfDetails />,
  },
]);

function App() {
  const prefersDarkMode = useMediaQuery('(prefers-color-scheme: dark)');

  const theme = useMemo(
    () => createTheme({
      palette: {
        mode: prefersDarkMode ? 'dark' : 'light',
      },
    }),
    [prefersDarkMode],
  );

  const [loading, setLoading] = useState(false);
  const loadingContext = useMemo(() => ({ loading, setLoading }), [loading, setLoading]);

  const [snackbarInfo, setSnackbarInfo] = useState<{
    message: string,
    severity: AlertColor,
    potentialUserChange: boolean,
  } | undefined>();
  const snackbarContext = useMemo(
    () => ({ status: snackbarInfo, setStatus: setSnackbarInfo }),
    [snackbarInfo, setSnackbarInfo],
  );

  return (
    <ThemeProvider theme={theme}>
      <LoadingContext.Provider value={loadingContext}>
        <SnackbarContext.Provider value={snackbarContext}>
          <CssBaseline />
          <main style={{ maxHeight: 'calc(100vh - 70px)', overflowY: 'auto' }}>
            <RouterProvider router={browserRouter} />

            <Container sx={{ marginTop: 4 }}>
              <Stack direction="row" justifyContent="center" flexWrap="wrap">
                <Link href="https://www.innosale.eu/" target="_blank" rel="noopener noreferrer">
                  <img
                    src={new URL('../innosale-logo.png', import.meta.url).toString()}
                    alt="InnoSale Logo"
                    height="40"
                    style={{
                      display: 'inline-block', borderRadius: 4, margin: 2, boxShadow: '0 0 5px 0 rgba(0,0,0,0.4)', padding: 8, background: '#FFFFFF',
                    }}
                  />
                </Link>
                <Link href="https://scads.ai/" target="_blank" rel="noopener noreferrer">
                  <img
                    src={new URL('../scads-logo.png', import.meta.url).toString()}
                    alt="Scads.AI Logo"
                    height="40"
                    style={{
                      display: 'inline-block', borderRadius: 4, margin: 2, boxShadow: '0 0 5px 0 rgba(0,0,0,0.4)', padding: 2, background: '#FFFFFF',
                    }}
                  />
                </Link>
                <Link href="https://secai.org/" target="_blank" rel="noopener noreferrer">
                  <img
                    src={new URL('../secai-logo.png', import.meta.url).toString()}
                    alt="Secai Logo"
                    height="40"
                    style={{
                      display: 'inline-block', borderRadius: 4, margin: 2, boxShadow: '0 0 5px 0 rgba(0,0,0,0.4)',
                    }}
                  />
                </Link>
                <Link href="https://perspicuous-computing.science" target="_blank" rel="noopener noreferrer">
                  <img
                    src={new URL('../cpec-logo.png', import.meta.url).toString()}
                    alt="CPEC Logo"
                    height="40"
                    style={{
                      display: 'inline-block', borderRadius: 4, margin: 2, boxShadow: '0 0 5px 0 rgba(0,0,0,0.4)', padding: 8, background: '#FFFFFF',
                    }}
                  />
                </Link>
                <Link href="https://iccl.inf.tu-dresden.de" target="_blank" rel="noopener noreferrer">
                  <img
                    src={new URL('../iccl-logo.png', import.meta.url).toString()}
                    alt="ICCL Logo"
                    height="40"
                    style={{
                      display: 'inline-block', borderRadius: 4, margin: 2, boxShadow: '0 0 5px 0 rgba(0,0,0,0.4)', padding: 4, background: '#FFFFFF',
                    }}
                  />
                </Link>
                <Link href="https://tu-dresden.de" target="_blank" rel="noopener noreferrer">
                  <img
                    src={new URL('../tud-logo.png', import.meta.url).toString()}
                    alt="TU Dresden Logo"
                    height="40"
                    style={{
                      display: 'inline-block', borderRadius: 4, margin: 2, boxShadow: '0 0 5px 0 rgba(0,0,0,0.4)',
                    }}
                  />
                </Link>
              </Stack>
            </Container>
          </main>

          <Footer />

          <Backdrop
            open={loading}
          >
            <CircularProgress color="inherit" />
          </Backdrop>
          <Snackbar
            open={!!snackbarInfo}
            autoHideDuration={10000}
            onClose={() => setSnackbarInfo(undefined)}
          >
            <Alert severity={snackbarInfo?.severity}>{snackbarInfo?.message}</Alert>
          </Snackbar>
        </SnackbarContext.Provider>
      </LoadingContext.Provider>
    </ThemeProvider>
  );
}

export default App;
