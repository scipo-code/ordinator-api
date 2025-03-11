import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import App from './App.tsx'
import './index.css'
import Layout from './Layout.tsx';


async function enableMocking() {
  if (import.meta.env.MODE !== "development") {
    return
  }

  const { worker } = await import('./mocks/browser.ts')

  return worker.start()
}


const queryClient = new QueryClient();

enableMocking().then(() => {
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <Layout>
          <App />
        </Layout>
      </QueryClientProvider>
    </StrictMode>
  )
})
