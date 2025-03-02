import { HashRouter, Route, Routes } from "react-router-dom";
import AssetDashboard from "./pages/AssetDashboard"; 
import NotFound from "./pages/NotFound"; 
import "./App.css";

function App() {
  return (
      <HashRouter basename="/">
        <Routes>
          <Route path="/dashboard/:asset" element={<AssetDashboard />} />
          <Route path="*" element={<NotFound />} />
        </Routes>
      </HashRouter>
  );
}

export default App;
