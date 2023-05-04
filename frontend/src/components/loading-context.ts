import { createContext } from 'react';

interface ILoadingContext {
  loading: boolean;
  setLoading: (loading: boolean) => void;
}

const LoadingContext = createContext<ILoadingContext>({
  loading: false,
  setLoading: () => {},
});

export default LoadingContext;
