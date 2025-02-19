import { HashRouter, Route, Routes } from "react-router-dom";
import Index from "./pages/Index";
import AssetPage from "./pages/AssetPage"; 
import NotFound from "./pages/NotFound"; 
import "./App.css";

function App() {
  return (
      <HashRouter basename="/">
        <Routes>
          <Route path="/" element={<Index />} />
          <Route path="/dashboard/:asset" element={<AssetPage />} />
          <Route path="*" element={<NotFound />} />
        </Routes>
      </HashRouter>
  );
}

export default App;
