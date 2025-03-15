import { HashRouter, Route, Routes } from "react-router-dom";
import PlanningView from "./pages/AssetDashboard"; 
// import NotFound from "./pages/NotFound"; 
import "./App.css";
import Layout from "./Layout";

function App() {
  return (
      <HashRouter basename="/">
        <Layout>
          <Routes>
            <Route path="/:asset" element={<PlanningView />} />
          </Routes>
        </Layout>
      </HashRouter>
  );
}

export default App;
