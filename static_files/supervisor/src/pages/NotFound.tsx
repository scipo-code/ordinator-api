import { Link } from 'react-router-dom';
import './NotFound.css';
import { buttonVariants } from '@/components/ui/button.tsx';


export default function NotFound() {
  return (
    <div className="not-found-container">
      <h1 className="not-found-title">404</h1>
      <p className="not-found-message">Oops! The page you're looking for doesn't exist.</p>
      <Link className={`${buttonVariants({
        variant: "outline_dark",
        size: "lg"
      })} custom-outline-button`} to="/">
        Go Back Home
      </Link>
    </div>
  );
};



