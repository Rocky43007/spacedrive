import { BloomOne } from '@sd/assets/images';
import { SdIntro } from '@sd/assets/videos';
import clsx from 'clsx';
import { AnimatePresence, motion } from 'framer-motion';
import { useState } from 'react';
import { Navigate, Outlet } from 'react-router';
import { useDebugState } from '@sd/client';
import DragRegion from '~/components/DragRegion';
import { useWindowSize } from '~/hooks';
import { useOperatingSystem } from '~/hooks/useOperatingSystem';

import DebugPopover from '../$libraryId/Layout/Sidebar/DebugPopover';
import { macOnly } from '../$libraryId/Layout/Sidebar/helpers';
import { OnboardingContext, useContextValue } from './context';
import Progress from './Progress';

export const Component = () => {
	const os = useOperatingSystem(false);
	const debugState = useDebugState();
	// FIX-ME: Intro video breaks onboarding for the web and Linux versions
	const [showIntro, setShowIntro] = useState(os === 'macOS' || os === 'windows');
	const windowSize = useWindowSize();

	const ctx = useContextValue();

	if (ctx.libraries.isLoading) return null;
	if (ctx.library?.uuid !== undefined) return <Navigate to={`/${ctx.library.uuid}`} replace />;

	return (
		<OnboardingContext.Provider value={ctx}>
			<div
				className={clsx(
					macOnly(os, 'bg-opacity-[0.75]'),
					'flex h-screen flex-col bg-sidebar text-ink'
				)}
			>
				<AnimatePresence>
					{showIntro && (
						<motion.div
							initial={{ opacity: 1 }}
							animate={{ opacity: 1 }}
							transition={{ duration: 0.5 }}
							exit={{ opacity: 0 }}
							className="absolute top-0 left-0 z-50 flex items-center justify-center w-screen h-screen"
						>
							{/*This makes sure on initial render a BG is visible before video loads*/}
							<svg
								width="100%"
								height="100%"
								className="absolute left-0 top-0 z-[-1]"
								viewBox={`0 0 ${windowSize.width} ${windowSize.height}`}
								fill="none"
								xmlns="http://www.w3.org/2000/svg"
							>
								<rect width="100%" height="100%" fill="#13151A" />
							</svg>
							<video
								className="relative z-10 mx-auto brightness-100"
								autoPlay
								onEnded={() => {
									setShowIntro(false);
								}}
								muted
								controls={false}
								src={SdIntro}
							/>
						</motion.div>
					)}
				</AnimatePresence>
				<DragRegion className="z-50 h-9" />
				<div className="flex flex-col gap-8 p-10 -mt-5 grow">
					<div className="flex flex-col items-center justify-center grow">
						<Outlet />
					</div>
					<Progress />
				</div>
				<div className="flex justify-center p-4">
					<p className="text-xs opacity-50 text-ink-dull">
						&copy; {new Date().getFullYear()} Spacedrive Technology Inc.
					</p>
				</div>
				<div className="absolute -z-10">
					<div className="relative w-screen h-screen">
						<img src={BloomOne} className="absolute size-[2000px]" />
						{/* <img src={BloomThree} className="absolute w-[2000px] h-[2000px] -right-[200px]" /> */}
					</div>
				</div>
				{debugState.enabled && <DebugPopover />}
			</div>
		</OnboardingContext.Provider>
	);
};
