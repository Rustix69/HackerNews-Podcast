import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { Card } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ThemeToggle } from '@/components/theme-toggle';
import { ExternalLink, Clock, ChevronLeft, ChevronRight } from 'lucide-react';

interface HNStory {
  id: number;
  title: string;
  url?: string;
  score: number;
  by: string;
  time: number;
  descendants?: number;
}

const HomePage = () => {
  const [allStories, setAllStories] = useState<HNStory[]>([]);
  const [loading, setLoading] = useState(true);
  const [currentPage, setCurrentPage] = useState(1);
  const storiesPerPage = 10;

  useEffect(() => {
    const fetchTopStories = async () => {
      try {
        // Fetch stories from our backend
        const response = await fetch('http://localhost:3001/api/stories');
        const storiesData = await response.json();
        
        setAllStories(storiesData.filter(story => story && story.title));
      } catch (error) {
        console.error('Error fetching stories:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchTopStories();
  }, []);

  const formatTimeAgo = (timestamp: number) => {
    const now = Date.now() / 1000;
    const diff = now - timestamp;
    const hours = Math.floor(diff / 3600);
    if (hours < 24) {
      return `${hours}h ago`;
    }
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  };

  // Calculate pagination
  const totalPages = Math.ceil(allStories.length / storiesPerPage);
  const startIndex = (currentPage - 1) * storiesPerPage;
  const endIndex = startIndex + storiesPerPage;
  const currentStories = allStories.slice(startIndex, endIndex);

  const goToPage = (page: number) => {
    setCurrentPage(page);
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-background">
        {/* Theme Toggle */}
        <div className="fixed top-4 right-4 z-10">
          <ThemeToggle />
        </div>
        
        <div className="container mx-auto px-4 py-8">
          <div className="text-center mb-12">
            <Skeleton className="h-12 w-96 mx-auto mb-4" />
            <Skeleton className="h-6 w-64 mx-auto" />
          </div>
          <div className="max-w-4xl mx-auto space-y-4">
            {Array.from({ length: 10 }).map((_, i) => (
              <Card key={i} className="p-6">
                <Skeleton className="h-6 w-full mb-2" />
                <Skeleton className="h-4 w-48" />
              </Card>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Theme Toggle */}
      <div className="fixed top-4 right-4 z-10">
        <ThemeToggle />
      </div>
      
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <div className="text-center mb-12">
          <h1 className="text-5xl font-bold text-foreground mb-4">
            HN Digest Podcast
          </h1>
          <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
            Transform top Hacker News stories into personalized podcasts. 
            Discover, configure, and generate your perfect tech digest.
          </p>
        </div>

        {/* Stories List */}
        <div className="max-w-4xl mx-auto">
          <div className="mb-6">
            <h2 className="text-2xl font-semibold text-foreground mb-2">
              Top Stories Today
            </h2>
            <div className="flex items-center justify-between">
              <p className="text-muted-foreground">
                Click any story to configure your podcast settings
              </p>
              <p className="text-sm text-muted-foreground">
                Page {currentPage} of {totalPages} • Showing {currentStories.length} stories
              </p>
            </div>
          </div>
          
          <div className="space-y-4">
            {currentStories.map((story, index) => {
              const globalIndex = startIndex + index;
              return (
                <Card key={story.id} className="hover:shadow-lg transition-shadow duration-200">
                <Link to={`/story/${story.id}`} className="block p-6">
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-2">
                        <Badge variant="secondary" className="text-xs">
                          #{globalIndex + 1}
                        </Badge>
                        <div className="flex items-center gap-2 text-sm text-muted-foreground">
                          <Clock className="w-3 h-3" />
                          {formatTimeAgo(story.time)}
                        </div>
                      </div>
                      
                      <h3 className="text-lg font-medium text-foreground mb-2 leading-tight">
                        {story.title}
                      </h3>
                      
                      <div className="flex items-center gap-4 text-sm text-muted-foreground">
                        <span>{story.score} points</span>
                        <span>by {story.by}</span>
                        {story.descendants !== undefined && (
                          <span>{story.descendants} comments</span>
                        )}
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-2">
                      {story.url && (
                        <ExternalLink className="w-4 h-4 text-muted-foreground" />
                      )}
                      <div className="text-sm text-podcast-orange font-medium">
                        Generate Podcast →
                      </div>
                    </div>
                  </div>
                </Link>
              </Card>
              );
            })}
          </div>

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="flex items-center justify-center gap-2 mt-8">
              <Button
                variant="outline"
                size="sm"
                onClick={() => goToPage(currentPage - 1)}
                disabled={currentPage === 1}
              >
                <ChevronLeft className="w-4 h-4 mr-1" />
                Previous
              </Button>
              
              <div className="flex items-center gap-1">
                {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                  let pageNum;
                  if (totalPages <= 5) {
                    pageNum = i + 1;
                  } else if (currentPage <= 3) {
                    pageNum = i + 1;
                  } else if (currentPage >= totalPages - 2) {
                    pageNum = totalPages - 4 + i;
                  } else {
                    pageNum = currentPage - 2 + i;
                  }
                  
                  return (
                    <Button
                      key={pageNum}
                      variant={currentPage === pageNum ? "default" : "outline"}
                      size="sm"
                      onClick={() => goToPage(pageNum)}
                      className="w-8 h-8 p-0"
                    >
                      {pageNum}
                    </Button>
                  );
                })}
              </div>
              
              <Button
                variant="outline"
                size="sm"
                onClick={() => goToPage(currentPage + 1)}
                disabled={currentPage === totalPages}
              >
                Next
                <ChevronRight className="w-4 h-4 ml-1" />
              </Button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default HomePage;