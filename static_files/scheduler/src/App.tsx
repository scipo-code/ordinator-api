import { HashRouter, Route, Routes } from "react-router-dom";
import AssetDashboard from "./pages/AssetDashboard"; 
// import NotFound from "./pages/NotFound"; 
import "./App.css";
import Layout from "./Layout";

function App() {
  return (
      <HashRouter basename="/">
        <Layout>
          <Routes>
            <Route path="/dashboard/:asset" element={<AssetDashboard />} />
          </Routes>
        </Layout>
      </HashRouter>
  );
}

export default App;
